#!/usr/bin/env python3
"""IMPALA-style async actor-learner training for the Decker gauntlet.

Multiple actor processes (CPU) collect trajectories from the Rust game engine
while a single learner process (GPU) trains continuously with V-trace
off-policy correction.

Usage:
    # Combat only (default 4 actors, 16 envs each)
    python scripts/train_impala.py --subclass dueling --combat-only

    # Tune actor/env counts
    python scripts/train_impala.py --subclass dueling --combat-only \\
        --n-actors 8 --envs-per-actor 32

    # Both stages
    python scripts/train_impala.py --subclass dueling

Key design:
  - Actors: CPU inference with stale model + VecGauntletEnv batched stepping
  - Learner: GPU V-trace updates, publishes weights via shared memory
  - Phase filtering: only target phases enter the training buffer
  - V-trace corrects for policy lag between actors and learner
"""

import argparse
import csv
import json
import os
import sys
import time
import uuid
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional

import numpy as np
import torch
import torch.multiprocessing as mp
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim


def write_flywheel_termination(reason: str = "normal") -> None:
    """Announce successful completion to the Flywheel sidecar if mounted."""
    termination = Path("/flywheel/termination")
    if termination.parent.exists():
        termination.write_text(reason, encoding="utf-8")


# ---------------------------------------------------------------------------
# Constants (imported inside functions for spawn compatibility)
# ---------------------------------------------------------------------------

PHASE_OFFSET = 8
PHASE_COUNT = 7


# ---------------------------------------------------------------------------
# Trajectory segment
# ---------------------------------------------------------------------------

@dataclass
class TrajectorySegment:
    """A fixed-length chunk of experience from one actor."""
    # Observations: dict of numpy arrays, each (seg_len, n_envs, ...)
    obs: dict
    # Actor outputs
    actions: np.ndarray          # (seg_len, n_envs) int32
    behaviour_log_probs: np.ndarray  # (seg_len, n_envs) float32
    values: np.ndarray           # (seg_len, n_envs) float32
    # Env outputs
    rewards: np.ndarray          # (seg_len, n_envs) float32
    dones: np.ndarray            # (seg_len, n_envs) float32
    # Bootstrap
    bootstrap_value: np.ndarray  # (n_envs,) float32
    # Metadata
    actor_id: int
    weight_version: int
    env_steps: int


# ---------------------------------------------------------------------------
# Shared-memory segment pool (zero-copy actor→learner transfer)
# ---------------------------------------------------------------------------

class SharedSegmentSlot:
    """One pre-allocated shared-memory buffer for a single trajectory segment.

    Actors write numpy data directly into shared memory; the learner reads
    it as numpy views — no pickling or copying.
    """

    def __init__(self, segment_length, envs_per_actor, dims):
        global_dim, card_feat_dim, enemy_feat_dim, action_feat_dim, _, max_hand, max_enemies, max_actions = dims
        S, E = segment_length, envs_per_actor

        # Pre-allocate shared arrays via multiprocessing.Array (backed by mmap).
        import ctypes
        def _shared_f32(size):
            return mp.Array(ctypes.c_float, size, lock=False)
        def _shared_i32(size):
            return mp.Array(ctypes.c_int, size, lock=False)

        # Obs arrays: (S, E, feat_dim) flattened.
        self._obs_global = _shared_f32(S * E * global_dim)
        self._obs_hand = _shared_f32(S * E * max_hand * card_feat_dim)
        self._obs_enemies = _shared_f32(S * E * max_enemies * enemy_feat_dim)
        self._obs_action_feats = _shared_f32(S * E * max_actions * action_feat_dim)
        self._obs_action_mask = _shared_f32(S * E * max_actions)
        self._obs_n_hand = _shared_i32(S * E)
        self._obs_n_enemies = _shared_i32(S * E)
        self._obs_n_actions = _shared_i32(S * E)

        # Actor outputs.
        self._actions = _shared_i32(S * E)
        self._behaviour_lp = _shared_f32(S * E)
        self._values = _shared_f32(S * E)

        # Env outputs.
        self._rewards = _shared_f32(S * E)
        self._dones = _shared_f32(S * E)

        # Bootstrap.
        self._bootstrap = _shared_f32(E)

        # Metadata.
        self._meta = _shared_i32(3)  # [actor_id, weight_version, env_steps]

        # Synchronization: 0=free, 1=ready for learner.
        self._ready = mp.Value(ctypes.c_int, 0, lock=False)

        # Store shapes for view construction.
        self._dims = dims
        self._S = S
        self._E = E

    def _np_f32(self, arr, shape):
        return np.frombuffer(arr, dtype=np.float32).reshape(shape)

    def _np_i32(self, arr, shape):
        return np.frombuffer(arr, dtype=np.int32).reshape(shape)

    def get_write_views(self):
        """Return numpy views into shared memory for the actor to fill."""
        S, E = self._S, self._E
        global_dim, card_feat_dim, enemy_feat_dim, action_feat_dim, _, max_hand, max_enemies, max_actions = self._dims
        return {
            "obs_global": self._np_f32(self._obs_global, (S, E, global_dim)),
            "obs_hand": self._np_f32(self._obs_hand, (S, E, max_hand, card_feat_dim)),
            "obs_enemies": self._np_f32(self._obs_enemies, (S, E, max_enemies, enemy_feat_dim)),
            "obs_action_feats": self._np_f32(self._obs_action_feats, (S, E, max_actions, action_feat_dim)),
            "obs_action_mask": self._np_f32(self._obs_action_mask, (S, E, max_actions)),
            "obs_n_hand": self._np_i32(self._obs_n_hand, (S, E)),
            "obs_n_enemies": self._np_i32(self._obs_n_enemies, (S, E)),
            "obs_n_actions": self._np_i32(self._obs_n_actions, (S, E)),
            "actions": self._np_i32(self._actions, (S, E)),
            "behaviour_lp": self._np_f32(self._behaviour_lp, (S, E)),
            "values": self._np_f32(self._values, (S, E)),
            "rewards": self._np_f32(self._rewards, (S, E)),
            "dones": self._np_f32(self._dones, (S, E)),
            "bootstrap": self._np_f32(self._bootstrap, (E,)),
            "meta": self._np_i32(self._meta, (3,)),
        }

    def get_read_views(self):
        """Return numpy views for the learner to read (same memory, zero-copy)."""
        return self.get_write_views()

    @property
    def ready(self):
        return self._ready.value == 1

    def mark_ready(self):
        self._ready.value = 1

    def mark_consumed(self):
        self._ready.value = 0


# ---------------------------------------------------------------------------
# V-trace
# ---------------------------------------------------------------------------

def vtrace_targets(
    behaviour_log_probs,  # (T, B)
    target_log_probs,     # (T, B)
    rewards,              # (T, B)
    values,               # (T, B)
    bootstrap_value,      # (B,)
    dones,                # (T, B)
    gamma=0.995,
    rho_bar=1.0,
    c_bar=1.0,
):
    """Compute V-trace value targets and policy gradient advantages."""
    with torch.no_grad():
        log_rhos = target_log_probs - behaviour_log_probs
        rhos = torch.exp(log_rhos)
        clipped_rhos = torch.clamp(rhos, max=rho_bar)
        cs = torch.clamp(rhos, max=c_bar)

        T, B = rewards.shape
        not_done = 1.0 - dones

        values_tp1 = torch.cat([values[1:], bootstrap_value.unsqueeze(0)], dim=0)
        deltas = clipped_rhos * (rewards + gamma * values_tp1 * not_done - values)

        vs_minus_v = torch.zeros(T, B, device=rewards.device)
        last = torch.zeros(B, device=rewards.device)
        for t in reversed(range(T)):
            last = deltas[t] + gamma * not_done[t] * cs[t] * last
            vs_minus_v[t] = last

        vs = vs_minus_v + values

        vs_tp1 = torch.cat([vs[1:], bootstrap_value.unsqueeze(0)], dim=0)
        advantages = clipped_rhos * (rewards + gamma * vs_tp1 * not_done - values)

        return vs, advantages


# ---------------------------------------------------------------------------
# Observation helpers (duplicated for spawn isolation)
# ---------------------------------------------------------------------------

def np_to_obs_batch(g, h, e, a, m, nh, ne, na, n_envs, dims):
    """Convert numpy arrays from VecGauntletEnv into a batched obs dict of tensors (CPU)."""
    _, card_feat_dim, enemy_feat_dim, action_feat_dim, _, max_hand, max_enemies, max_actions = dims

    return {
        "global": torch.from_numpy(g),
        "hand": torch.from_numpy(h).reshape(n_envs, max_hand, card_feat_dim),
        "enemies": torch.from_numpy(e).reshape(n_envs, max_enemies, enemy_feat_dim),
        "action_feats": torch.from_numpy(a).reshape(n_envs, max_actions, action_feat_dim),
        "action_mask": torch.from_numpy(m),
        "n_hand": torch.from_numpy(nh),
        "n_enemies": torch.from_numpy(ne),
        "n_actions": torch.from_numpy(na),
    }


def np_to_obs_batch_numpy(g, h, e, a, m, nh, ne, na, n_envs, dims):
    """Convert numpy arrays from VecGauntletEnv into a batched obs dict of numpy arrays."""
    _, card_feat_dim, enemy_feat_dim, action_feat_dim, _, max_hand, max_enemies, max_actions = dims

    return {
        "global": g,
        "hand": h.reshape(n_envs, max_hand, card_feat_dim),
        "enemies": e.reshape(n_envs, max_enemies, enemy_feat_dim),
        "action_feats": a.reshape(n_envs, max_actions, action_feat_dim),
        "action_mask": m,
        "n_hand": nh,
        "n_enemies": ne,
        "n_actions": na,
    }


# ---------------------------------------------------------------------------
# Actor process
# ---------------------------------------------------------------------------

def actor_fn(
    actor_id,
    subclass,
    race,
    synergy_group,
    envs_per_actor,
    segment_length,
    dims,
    train_phase_ids_list,  # list, not set (for pickling)
    reward_mode,
    model_non_trainable,
    shared_model_state,    # shared memory state dict
    weight_version,        # mp.Value
    segment_slot,          # SharedSegmentSlot (shared memory)
    stop_event,            # mp.Event
    seed,
    gamma,
    model_kwargs,          # dict of DeckModel constructor args
):
    """Actor process: collect trajectories and push to queue."""
    # Imports inside function for Windows spawn compatibility.
    scripts_dir = str(Path(__file__).resolve().parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)

    import decker_pyenv as decker
    from model import DeckModel

    train_phase_ids = set(train_phase_ids_list)
    train_phase_arr = np.array(sorted(train_phase_ids), dtype=np.int32)

    # Create local model on CPU.
    local_model = DeckModel(**model_kwargs)
    local_model.load_state_dict(shared_model_state)
    local_model.eval()
    local_weight_version = weight_version.value

    # Create vectorized environment.
    rng = np.random.RandomState(seed)
    seeds = [int(rng.randint(0, 2**31)) for _ in range(envs_per_actor)]
    vec_env = decker.VecGauntletEnv(subclass, seeds, race,
                                     synergy_filter=synergy_group)

    # Initial observations.
    g, h, e, a, m, nh, ne, na = vec_env.get_obs_all()
    obs_batch = np_to_obs_batch(g, h, e, a, m, nh, ne, na, envs_per_actor, dims)

    n_envs = envs_per_actor
    pending_reward = np.zeros(n_envs, dtype=np.float64)

    while not stop_event.is_set():
        # Check for weight update.
        current_wv = weight_version.value
        if current_wv != local_weight_version:
            local_model.load_state_dict(shared_model_state)
            local_weight_version = current_wv

        # Collect one segment of segment_length trainable steps per env.
        buf_obs = {
            "global": [], "hand": [], "enemies": [], "action_feats": [],
            "action_mask": [], "n_hand": [], "n_enemies": [], "n_actions": [],
        }
        buf_actions = [[] for _ in range(n_envs)]
        buf_log_probs = [[] for _ in range(n_envs)]
        buf_values = [[] for _ in range(n_envs)]
        buf_rewards = [[] for _ in range(n_envs)]
        buf_dones = [[] for _ in range(n_envs)]

        buf_count = np.zeros(n_envs, dtype=np.int32)
        total_env_steps = 0

        while buf_count.min() < segment_length:
            if stop_event.is_set():
                return

            # CPU inference.
            with torch.no_grad():
                act_t, lp_t, _, val_t = local_model.get_action_and_value(obs_batch)
            actions_np = act_t.numpy().copy()
            lp_np = lp_t.numpy().copy()
            val_np = val_t.numpy().copy()

            # Phase check: random for non-trainable, non-target phases.
            cur_phases = obs_batch["global"][:, PHASE_OFFSET:PHASE_OFFSET + PHASE_COUNT].argmax(dim=-1).numpy()
            na_np = obs_batch["n_actions"].numpy()

            for ei in range(n_envs):
                n_legal = int(na_np[ei])
                if not model_non_trainable and cur_phases[ei] not in train_phase_ids:
                    actions_np[ei] = np.random.randint(0, max(n_legal, 1))
                    lp_np[ei] = 0.0
                    val_np[ei] = 0.0
                elif actions_np[ei] >= n_legal:
                    actions_np[ei] = 0

            # Step all environments.
            (g, h, e, a, m, nh, ne, na,
             raw_rewards, dones_np, _fights_won,
             phase_ids_np, fight_hp_np, fight_max_hp_np
             ) = vec_env.step_all(actions_np.tolist())
            total_env_steps += n_envs

            # Compute rewards.
            if reward_mode == "delta-hp":
                rewards_np = np.zeros(n_envs, dtype=np.float32)
                fight_ended = fight_max_hp_np > 0
                won = fight_ended & (fight_hp_np > 0)
                lost = fight_ended & (fight_hp_np <= 0)
                rewards_np[won] = np.maximum(
                    fight_hp_np[won] / np.maximum(fight_max_hp_np[won], 1.0),
                    0.01,
                )
                rewards_np[lost] = -1.0
            else:
                rewards_np = raw_rewards

            # Phase filtering.
            trainable_mask = np.isin(phase_ids_np, train_phase_arr)

            for ei in range(n_envs):
                is_trainable = trainable_mask[ei]
                needs_more = buf_count[ei] < segment_length
                should_record = is_trainable and needs_more
                done = dones_np[ei] > 0.5
                reward = float(rewards_np[ei])

                if should_record:
                    # Record pre-step obs for this env at this timestep.
                    for k in buf_obs:
                        buf_obs[k].append((ei, buf_count[ei], obs_batch[k][ei].numpy().copy()))
                    buf_actions[ei].append(int(actions_np[ei]))
                    buf_log_probs[ei].append(float(lp_np[ei]))
                    buf_values[ei].append(float(val_np[ei]))
                    buf_rewards[ei].append(pending_reward[ei] + reward)
                    buf_dones[ei].append(1.0 if done else 0.0)
                    pending_reward[ei] = 0.0
                    buf_count[ei] += 1
                else:
                    pending_reward[ei] += reward
                    if done:
                        if buf_rewards[ei]:
                            buf_rewards[ei][-1] += pending_reward[ei]
                            buf_dones[ei][-1] = 1.0
                        pending_reward[ei] = 0.0

            # Convert new obs.
            obs_batch = np_to_obs_batch(g, h, e, a, m, nh, ne, na, n_envs, dims)

        # Flush pending rewards.
        for ei in range(n_envs):
            if pending_reward[ei] != 0.0 and buf_rewards[ei]:
                buf_rewards[ei][-1] += pending_reward[ei]
                pending_reward[ei] = 0.0

        # Bootstrap value.
        with torch.no_grad():
            bootstrap_val = local_model.get_value(obs_batch).numpy().copy()

        # Wait for the slot to be consumed before overwriting.
        while not stop_event.is_set() and segment_slot.ready:
            time.sleep(0.001)
        if stop_event.is_set():
            return

        # Write directly into shared memory (zero-copy).
        views = segment_slot.get_write_views()

        # Zero out first (shared memory persists between uses).
        for v in views.values():
            v[:] = 0

        # Obs.
        obs_key_map = {
            "global": "obs_global", "hand": "obs_hand", "enemies": "obs_enemies",
            "action_feats": "obs_action_feats", "action_mask": "obs_action_mask",
            "n_hand": "obs_n_hand", "n_enemies": "obs_n_enemies", "n_actions": "obs_n_actions",
        }
        for k, shm_k in obs_key_map.items():
            for ei_val, t_val, data in buf_obs[k]:
                views[shm_k][t_val, ei_val] = data

        # Actor outputs + env outputs.
        for ei in range(n_envs):
            for t in range(segment_length):
                views["actions"][t, ei] = buf_actions[ei][t]
                views["behaviour_lp"][t, ei] = buf_log_probs[ei][t]
                views["values"][t, ei] = buf_values[ei][t]
                views["rewards"][t, ei] = buf_rewards[ei][t]
                views["dones"][t, ei] = buf_dones[ei][t]

        views["bootstrap"][:] = bootstrap_val
        views["meta"][0] = actor_id
        views["meta"][1] = local_weight_version
        views["meta"][2] = total_env_steps

        # Signal the learner.
        segment_slot.mark_ready()


# ---------------------------------------------------------------------------
# Numpy actor process
# ---------------------------------------------------------------------------

def numpy_actor_fn(
    actor_id,
    subclass,
    race,
    synergy_group,
    envs_per_actor,
    segment_length,
    dims,
    train_phase_ids_list,
    reward_mode,
    model_non_trainable,
    shared_weight_arrays,  # dict of shared mp.Array buffers
    weight_version,
    segment_slot,
    stop_event,
    seed,
    gamma,
    model_kwargs,
):
    """Actor process using numpy inference (no torch)."""
    scripts_dir = str(Path(__file__).resolve().parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)

    import decker_pyenv as decker
    from numpy_eval import NumpyModel

    train_phase_ids = set(train_phase_ids_list)
    train_phase_arr = np.array(sorted(train_phase_ids), dtype=np.int32)

    # Build numpy state_dict from shared memory arrays.
    def _read_shared_weights():
        sd = {}
        for key, (arr, shape) in shared_weight_arrays.items():
            sd[key] = np.frombuffer(arr, dtype=np.float32).reshape(shape).copy()
        return sd

    local_model = NumpyModel(_read_shared_weights(), model_kwargs['vocab_size'])
    local_weight_version = weight_version.value

    # Seed numpy RNG for this actor.
    np.random.seed(seed)

    # Create vectorized environment.
    rng = np.random.RandomState(seed)
    seeds = [int(rng.randint(0, 2**31)) for _ in range(envs_per_actor)]
    vec_env = decker.VecGauntletEnv(subclass, seeds, race,
                                     synergy_filter=synergy_group)

    g, h, e, a, m, nh, ne, na = vec_env.get_obs_all()
    obs_batch = np_to_obs_batch_numpy(g, h, e, a, m, nh, ne, na, envs_per_actor, dims)

    n_envs = envs_per_actor
    pending_reward = np.zeros(n_envs, dtype=np.float64)

    while not stop_event.is_set():
        # Check for weight update.
        current_wv = weight_version.value
        if current_wv != local_weight_version:
            local_model.load_state_dict(_read_shared_weights())
            local_weight_version = current_wv

        buf_obs = {
            "global": [], "hand": [], "enemies": [], "action_feats": [],
            "action_mask": [], "n_hand": [], "n_enemies": [], "n_actions": [],
        }
        buf_actions = [[] for _ in range(n_envs)]
        buf_log_probs = [[] for _ in range(n_envs)]
        buf_values = [[] for _ in range(n_envs)]
        buf_rewards = [[] for _ in range(n_envs)]
        buf_dones = [[] for _ in range(n_envs)]

        buf_count = np.zeros(n_envs, dtype=np.int32)
        total_env_steps = 0

        while buf_count.min() < segment_length:
            if stop_event.is_set():
                return

            # Numpy inference.
            actions_np, lp_np, val_np = local_model.get_action_and_value_batch(obs_batch)

            # Phase check.
            cur_phases = obs_batch["global"][:, PHASE_OFFSET:PHASE_OFFSET + PHASE_COUNT].argmax(axis=-1)
            na_np = obs_batch["n_actions"]

            for ei in range(n_envs):
                n_legal = int(na_np[ei])
                if not model_non_trainable and cur_phases[ei] not in train_phase_ids:
                    actions_np[ei] = np.random.randint(0, max(n_legal, 1))
                    lp_np[ei] = 0.0
                    val_np[ei] = 0.0
                elif actions_np[ei] >= n_legal:
                    actions_np[ei] = 0

            # Step all environments.
            (g, h, e, a, m, nh, ne, na,
             raw_rewards, dones_np, _fights_won,
             phase_ids_np, fight_hp_np, fight_max_hp_np
             ) = vec_env.step_all(actions_np.tolist())
            total_env_steps += n_envs

            # Compute rewards.
            if reward_mode == "delta-hp":
                rewards_np = np.zeros(n_envs, dtype=np.float32)
                fight_ended = fight_max_hp_np > 0
                won = fight_ended & (fight_hp_np > 0)
                lost = fight_ended & (fight_hp_np <= 0)
                rewards_np[won] = np.maximum(
                    fight_hp_np[won] / np.maximum(fight_max_hp_np[won], 1.0),
                    0.01,
                )
                rewards_np[lost] = -1.0
            else:
                rewards_np = raw_rewards

            # Phase filtering.
            trainable_mask = np.isin(phase_ids_np, train_phase_arr)

            for ei in range(n_envs):
                is_trainable = trainable_mask[ei]
                needs_more = buf_count[ei] < segment_length
                should_record = is_trainable and needs_more
                done = dones_np[ei] > 0.5
                reward = float(rewards_np[ei])

                if should_record:
                    for k in buf_obs:
                        buf_obs[k].append((ei, buf_count[ei], obs_batch[k][ei].copy()))
                    buf_actions[ei].append(int(actions_np[ei]))
                    buf_log_probs[ei].append(float(lp_np[ei]))
                    buf_values[ei].append(float(val_np[ei]))
                    buf_rewards[ei].append(pending_reward[ei] + reward)
                    buf_dones[ei].append(1.0 if done else 0.0)
                    pending_reward[ei] = 0.0
                    buf_count[ei] += 1
                else:
                    pending_reward[ei] += reward
                    if done:
                        if buf_rewards[ei]:
                            buf_rewards[ei][-1] += pending_reward[ei]
                            buf_dones[ei][-1] = 1.0
                        pending_reward[ei] = 0.0

            obs_batch = np_to_obs_batch_numpy(g, h, e, a, m, nh, ne, na, n_envs, dims)

        # Flush pending rewards.
        for ei in range(n_envs):
            if pending_reward[ei] != 0.0 and buf_rewards[ei]:
                buf_rewards[ei][-1] += pending_reward[ei]
                pending_reward[ei] = 0.0

        # Bootstrap value.
        bootstrap_val = local_model.get_value_batch(obs_batch)

        # Wait for slot.
        while not stop_event.is_set() and segment_slot.ready:
            time.sleep(0.001)
        if stop_event.is_set():
            return

        # Write to shared memory.
        views = segment_slot.get_write_views()
        for v in views.values():
            v[:] = 0

        obs_key_map = {
            "global": "obs_global", "hand": "obs_hand", "enemies": "obs_enemies",
            "action_feats": "obs_action_feats", "action_mask": "obs_action_mask",
            "n_hand": "obs_n_hand", "n_enemies": "obs_n_enemies", "n_actions": "obs_n_actions",
        }
        for k, shm_k in obs_key_map.items():
            for ei_val, t_val, data in buf_obs[k]:
                views[shm_k][t_val, ei_val] = data

        for ei in range(n_envs):
            for t in range(segment_length):
                views["actions"][t, ei] = buf_actions[ei][t]
                views["behaviour_lp"][t, ei] = buf_log_probs[ei][t]
                views["values"][t, ei] = buf_values[ei][t]
                views["rewards"][t, ei] = buf_rewards[ei][t]
                views["dones"][t, ei] = buf_dones[ei][t]

        views["bootstrap"][:] = bootstrap_val
        views["meta"][0] = actor_id
        views["meta"][1] = local_weight_version
        views["meta"][2] = total_env_steps

        segment_slot.mark_ready()


# ---------------------------------------------------------------------------
# Evaluation
# ---------------------------------------------------------------------------



# ---------------------------------------------------------------------------
# Plotting
# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

def parse_args():
    parser = argparse.ArgumentParser(
        description="IMPALA-style async training for Decker gauntlet",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Examples:
  python scripts/train_impala.py --subclass dueling --combat-only
  python scripts/train_impala.py --subclass dueling --n-actors 8 --envs-per-actor 32
""",
    )
    parser.add_argument("--subclass", default=None,
                        help="Fighter sub-class (two_handed, defense, dueling)")
    parser.add_argument("--race", default="human")
    parser.add_argument("--synergy-group", default=None)

    stage_group = parser.add_mutually_exclusive_group()
    stage_group.add_argument("--combat-only", action="store_true")
    stage_group.add_argument("--curation-only", action="store_true")
    stage_group.add_argument("--shared", action="store_true")

    parser.add_argument("--combat-timesteps", type=int, default=20_000_000)
    parser.add_argument("--curation-timesteps", type=int, default=10_000_000)
    parser.add_argument("--shared-timesteps", type=int, default=10_000_000)
    parser.add_argument("--reward-mode", default="delta-hp",
                        choices=["fights-won", "delta-hp"])

    # IMPALA-specific.
    parser.add_argument("--actor-backend", default="numpy",
                        choices=["numpy", "torch"],
                        help="Actor inference backend (default: numpy)")
    parser.add_argument("--n-actors", type=int, default=4)
    parser.add_argument("--envs-per-actor", type=int, default=16)
    parser.add_argument("--segment-length", type=int, default=32,
                        help="Trainable steps per trajectory segment per env")
    parser.add_argument("--queue-depth", type=int, default=16,
                        help="Max queued segments before actors block")
    parser.add_argument("--segments-per-update", type=int, default=4,
                        help="Segments to accumulate per learner update")

    # V-trace.
    parser.add_argument("--rho-bar", type=float, default=1.0)
    parser.add_argument("--c-bar", type=float, default=1.0)

    # Shared hyperparameters.
    parser.add_argument("--gamma", type=float, default=0.995)
    parser.add_argument("--lr", type=float, default=3e-4)
    parser.add_argument("--ent-coef", type=float, default=0.02)
    parser.add_argument("--vf-coef", type=float, default=0.5)
    parser.add_argument("--max-grad-norm", type=float, default=0.5)
    parser.add_argument("--anneal-lr", action=argparse.BooleanOptionalAction,
                        default=True)

    parser.add_argument("--checkpoint-dir", default="checkpoints")
    parser.add_argument("--seed", type=int, default=None)
    parser.add_argument("--device", default="auto", choices=["cpu", "cuda", "auto"])
    parser.add_argument("--init-model", default=None)
    parser.add_argument("--max-time", type=float, default=None,
                        help="Wall-clock time limit in seconds for the entire run. "
                             "Training self-terminates when exceeded.")
    return parser.parse_args()


# ---------------------------------------------------------------------------
# Learner loop
# ---------------------------------------------------------------------------

def run_impala_training(
    dims, args,
    train_phase_ids, reward_mode, model_non_trainable,
    total_timesteps, stage_label,
    train_device,
    run_dir, seed_rng,
    model_kwargs,
    deadline: float | None = None,
):
    """Run one IMPALA training stage. Returns path to best_model.pt."""
    from model import DeckModel
    from strategy import PHASE_NAMES
    import decker_pyenv as decker

    phase_names = [PHASE_NAMES[i] for i in sorted(train_phase_ids)]
    print(f"\n{'='*60}")
    print(f"STAGE: {stage_label} (IMPALA)")
    print(f"  Phases: {phase_names}")
    print(f"  Reward: {reward_mode}")
    print(f"  Non-trainable policy: {'model' if model_non_trainable else 'random'}")
    print(f"  Timesteps: {total_timesteps:,}")
    print(f"  Actors: {args.n_actors} x {args.envs_per_actor} envs")
    print(f"  Segment length: {args.segment_length}")
    print(f"  Run dir: {run_dir}")
    print(f"{'='*60}\n")

    # Create shared model (CPU, shared memory).
    shared_model = DeckModel(**model_kwargs)
    if args.init_model:
        ckpt = torch.load(args.init_model, map_location="cpu", weights_only=False)
        if isinstance(ckpt, dict) and "state_dict" in ckpt:
            shared_model.load_state_dict(ckpt["state_dict"])
        else:
            shared_model.load_state_dict(ckpt)
        print(f"Initialized from {args.init_model}")
    shared_state = shared_model.state_dict()
    # Put tensors in shared memory for cross-process access.
    for k, v in shared_state.items():
        shared_state[k] = v.share_memory_()

    # Learner model on GPU.
    learner_model = DeckModel(**model_kwargs)
    learner_model.load_state_dict(shared_state)
    learner_model.to(train_device)
    learner_model.train()

    # Freeze shared encoders and combat heads during curation-only training
    # so that curation gradients don't corrupt combat representations.
    _frozen_prefixes = ("global_enc.", "card_embed.", "card_enc.", "enemy_enc.",
                        "combat_action_scorer.", "combat_value_head.")
    if model_non_trainable:
        for name, param in learner_model.named_parameters():
            if any(name.startswith(p) for p in _frozen_prefixes):
                param.requires_grad = False

    optimizer = optim.Adam(
        filter(lambda p: p.requires_grad, learner_model.parameters()),
        lr=args.lr, eps=1e-5,
    )

    # Engine version.
    current_version = json.loads(decker.engine_version())
    os.makedirs(run_dir, exist_ok=True)
    (run_dir / "version.json").write_text(json.dumps(current_version, indent=2))

    stage_config = {
        "stage": stage_label,
        "algorithm": "impala",
        "train_phases": phase_names,
        "reward_mode": reward_mode,
        "model_non_trainable": model_non_trainable,
        "total_timesteps": total_timesteps,
        "n_actors": args.n_actors,
        "envs_per_actor": args.envs_per_actor,
        "segment_length": args.segment_length,
    }
    stage_config.update({k: v for k, v in vars(args).items()
                         if k not in ("combat_only", "curation_only", "combat_timesteps",
                                      "curation_timesteps", "init_model", "checkpoint_dir")})
    (run_dir / "config.json").write_text(json.dumps(stage_config, indent=2))

    # Logging.
    log_path = run_dir / "training_log.csv"
    log_file = open(log_path, "w", newline="")
    log_writer = csv.writer(log_file)
    log_writer.writerow([
        "timestep", "update", "policy_loss", "value_loss", "entropy",
        "mean_rho", "clip_frac", "queue_depth", "policy_lag", "elapsed_s",
    ])

    # Shared state for actor communication.
    weight_version = mp.Value('i', 0)
    stop_event = mp.Event()

    # Pre-allocate shared-memory segment slots (one per actor).
    segment_slots = [
        SharedSegmentSlot(args.segment_length, args.envs_per_actor, dims)
        for _ in range(args.n_actors)
    ]

    # For numpy actors: create shared numpy weight arrays.
    use_numpy_actors = getattr(args, 'actor_backend', 'numpy') == 'numpy'
    shared_weight_arrays = None
    if use_numpy_actors:
        import ctypes
        shared_weight_arrays = {}
        for k, v in shared_state.items():
            flat = v.numpy().ravel()
            arr = mp.Array(ctypes.c_float, len(flat), lock=False)
            np.frombuffer(arr, dtype=np.float32)[:] = flat
            shared_weight_arrays[k] = (arr, tuple(v.shape))

    # Spawn actors.
    synergy_group = getattr(args, 'synergy_group', None)
    race = getattr(args, 'race', 'human')
    actors = []
    for i in range(args.n_actors):
        actor_seed = int(seed_rng.randint(0, 2**31))
        if use_numpy_actors:
            target = numpy_actor_fn
            actor_args = (
                i, args.subclass, race, synergy_group,
                args.envs_per_actor, args.segment_length, dims,
                sorted(train_phase_ids),
                reward_mode, model_non_trainable,
                shared_weight_arrays, weight_version,
                segment_slots[i], stop_event,
                actor_seed, args.gamma, model_kwargs,
            )
        else:
            target = actor_fn
            actor_args = (
                i, args.subclass, race, synergy_group,
                args.envs_per_actor, args.segment_length, dims,
                sorted(train_phase_ids),
                reward_mode, model_non_trainable,
                shared_state, weight_version,
                segment_slots[i], stop_event,
                actor_seed, args.gamma, model_kwargs,
            )
        p = mp.Process(target=target, args=actor_args, daemon=True)
        p.start()
        actors.append(p)
        backend_label = "numpy" if use_numpy_actors else "torch"
        print(f"  Started {backend_label} actor {i} (pid={p.pid})")

    global_step = 0
    update_count = 0
    start_time = time.time()

    # Timing accumulators.
    t_dequeue = 0.0
    t_vtrace = 0.0
    t_gradient = 0.0
    t_publish = 0.0
    timing_updates = 0

    _, card_feat_dim, enemy_feat_dim, action_feat_dim, _, max_hand, max_enemies, max_actions = dims

    print(f"\nLearner running on {train_device}")
    print(f"Training for {total_timesteps:,} trainable timesteps")
    print()

    try:
        while global_step < total_timesteps:
            if deadline is not None and time.time() > deadline:
                print(f"\n[max-time] Wall-clock deadline reached. Stopping {stage_label}.")
                break
            # ── Collect ready segments from shared memory ─────────────
            _t0 = time.perf_counter()

            # Wait until at least one slot is ready.
            ready_indices = []
            while not ready_indices:
                if stop_event.is_set():
                    break
                ready_indices = [i for i, slot in enumerate(segment_slots) if slot.ready]
                if not ready_indices:
                    # Check if actors are alive.
                    alive = [p.is_alive() for p in actors]
                    if not any(alive):
                        print("ERROR: All actors died.")
                        break
                    time.sleep(0.0005)

            if not ready_indices:
                continue

            t_dequeue += time.perf_counter() - _t0

            # ── Assemble batch from shared memory (zero-copy reads) ──
            _t0 = time.perf_counter()

            seg_len = args.segment_length
            total_envs = len(ready_indices) * args.envs_per_actor

            # Read views and build tensors. We copy here to allow the
            # slot to be reused immediately after mark_consumed().
            obs_parts = {k: [] for k in ["global", "hand", "enemies", "action_feats",
                                          "action_mask", "n_hand", "n_enemies", "n_actions"]}
            actions_parts = []
            blp_parts = []
            rewards_parts = []
            dones_parts = []
            bootstrap_parts = []
            env_steps_total = 0
            weight_versions = []

            obs_key_map = {
                "global": "obs_global", "hand": "obs_hand", "enemies": "obs_enemies",
                "action_feats": "obs_action_feats", "action_mask": "obs_action_mask",
                "n_hand": "obs_n_hand", "n_enemies": "obs_n_enemies", "n_actions": "obs_n_actions",
            }

            for idx in ready_indices:
                views = segment_slots[idx].get_read_views()
                for k, shm_k in obs_key_map.items():
                    obs_parts[k].append(views[shm_k].copy())
                actions_parts.append(views["actions"].copy())
                blp_parts.append(views["behaviour_lp"].copy())
                rewards_parts.append(views["rewards"].copy())
                dones_parts.append(views["dones"].copy())
                bootstrap_parts.append(views["bootstrap"].copy())
                meta = views["meta"]
                weight_versions.append(int(meta[1]))
                env_steps_total += int(meta[2])
                # Release the slot for reuse.
                segment_slots[idx].mark_consumed()

            cat_obs = {}
            for k in obs_parts:
                cat_obs[k] = torch.from_numpy(np.concatenate(obs_parts[k], axis=1))
            cat_actions = torch.from_numpy(np.concatenate(actions_parts, axis=1)).long()
            cat_behaviour_lp = torch.from_numpy(np.concatenate(blp_parts, axis=1))
            cat_rewards = torch.from_numpy(np.concatenate(rewards_parts, axis=1))
            cat_dones = torch.from_numpy(np.concatenate(dones_parts, axis=1))
            cat_bootstrap = torch.from_numpy(np.concatenate(bootstrap_parts, axis=0))

            trainable_steps = seg_len * total_envs
            total_env_steps = env_steps_total
            max_policy_lag = max(
                weight_version.value - wv for wv in weight_versions
            )

            # Move to GPU.
            cat_obs_gpu = {k: v.to(train_device) for k, v in cat_obs.items()}
            cat_actions_gpu = cat_actions.to(train_device)
            cat_behaviour_lp_gpu = cat_behaviour_lp.to(train_device)
            cat_rewards_gpu = cat_rewards.to(train_device)
            cat_dones_gpu = cat_dones.to(train_device)
            cat_bootstrap_gpu = cat_bootstrap.to(train_device)

            # Reshape obs for model: flatten (seg_len, total_envs) -> (seg_len * total_envs, ...).
            flat_batch_size = seg_len * total_envs
            flat_obs = {}
            for k, v in cat_obs_gpu.items():
                flat_obs[k] = v.reshape(flat_batch_size, *v.shape[2:])
            flat_actions = cat_actions_gpu.reshape(flat_batch_size)

            # Forward pass to get current policy log_probs and values.
            _, target_log_probs_flat, entropy_flat, values_flat = \
                learner_model.get_action_and_value(flat_obs, flat_actions)

            # Reshape back to (seg_len, total_envs).
            target_log_probs = target_log_probs_flat.reshape(seg_len, total_envs)
            values = values_flat.reshape(seg_len, total_envs)
            entropy = entropy_flat.reshape(seg_len, total_envs)

            # Bootstrap value from learner's current model.
            with torch.no_grad():
                # Build obs dict for bootstrap step.
                bootstrap_obs = {}
                for k, v in cat_obs_gpu.items():
                    # Use last step's next obs (which is approximated by bootstrap from actor).
                    # Actually, we need the obs AFTER the last step. The actor computed
                    # bootstrap from its stale model; we recompute with the learner.
                    # For simplicity, use the actor's bootstrap (it's close enough
                    # when policy lag is small).
                    pass
                learner_bootstrap = cat_bootstrap_gpu

            # V-trace targets.
            vs, advantages = vtrace_targets(
                behaviour_log_probs=cat_behaviour_lp_gpu,
                target_log_probs=target_log_probs.detach(),
                rewards=cat_rewards_gpu,
                values=values.detach(),
                bootstrap_value=learner_bootstrap,
                dones=cat_dones_gpu,
                gamma=args.gamma,
                rho_bar=args.rho_bar,
                c_bar=args.c_bar,
            )

            # IS ratio stats for monitoring.
            with torch.no_grad():
                log_rhos = target_log_probs.detach() - cat_behaviour_lp_gpu
                rhos = torch.exp(log_rhos)
                mean_rho = rhos.mean().item()
                clip_frac = (rhos > args.rho_bar).float().mean().item()

            t_vtrace += time.perf_counter() - _t0

            # ── Gradient update ───────────────────────────────────────
            _t0 = time.perf_counter()

            # Policy gradient loss (IMPALA-style).
            # advantages are already rho-clipped from vtrace_targets.
            pg_loss = -(advantages.detach().reshape(-1) * target_log_probs_flat).mean()

            # Value loss.
            vf_loss = 0.5 * (values_flat - vs.detach().reshape(-1)).pow(2).mean()

            # Entropy bonus.
            entropy_loss = entropy_flat.mean()

            loss = pg_loss + args.vf_coef * vf_loss - args.ent_coef * entropy_loss

            if args.anneal_lr:
                frac = 1.0 - (global_step / total_timesteps)
                lr_now = args.lr * max(frac, 0.0)
                for param_group in optimizer.param_groups:
                    param_group["lr"] = lr_now

            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(learner_model.parameters(), args.max_grad_norm)
            optimizer.step()

            t_gradient += time.perf_counter() - _t0

            # ── Publish weights ───────────────────────────────────────
            _t0 = time.perf_counter()

            learner_state = learner_model.state_dict()
            for k in shared_state:
                shared_state[k].copy_(learner_state[k].cpu())
            # Also update numpy shared arrays if numpy actors are in use.
            if shared_weight_arrays is not None:
                for k, (arr, shape) in shared_weight_arrays.items():
                    np.frombuffer(arr, dtype=np.float32)[:] = (
                        learner_state[k].cpu().numpy().ravel())
            weight_version.value += 1

            t_publish += time.perf_counter() - _t0

            global_step += trainable_steps
            update_count += 1
            timing_updates += 1

            # ── Logging ───────────────────────────────────────────────
            elapsed = time.time() - start_time
            pg_val = pg_loss.item()
            vf_val = vf_loss.item()
            ent_val = entropy_loss.item()

            if update_count % 10 == 0:
                sps = global_step / elapsed
                qd = sum(1 for s in segment_slots if s.ready)
                print(
                    f"[{global_step:>9,}] "
                    f"pg={pg_val:+.4f} vf={vf_val:.4f} ent={ent_val:.3f} "
                    f"rho={mean_rho:.3f} clip={clip_frac:.2f} "
                    f"lag={max_policy_lag} q={qd}/{args.n_actors} "
                    f"sps={sps:.0f}"
                )

            if timing_updates > 0 and timing_updates % 50 == 0:
                t_total = t_dequeue + t_vtrace + t_gradient + t_publish
                if t_total > 0:
                    print(f"  TIMING ({timing_updates} updates, {t_total:.1f}s total):")
                    print(f"    dequeue:    {t_dequeue:6.1f}s  ({100*t_dequeue/t_total:4.1f}%)")
                    print(f"    vtrace:     {t_vtrace:6.1f}s  ({100*t_vtrace/t_total:4.1f}%)")
                    print(f"    gradient:   {t_gradient:6.1f}s  ({100*t_gradient/t_total:4.1f}%)")
                    print(f"    publish:    {t_publish:6.1f}s  ({100*t_publish/t_total:4.1f}%)")
                t_dequeue = t_vtrace = t_gradient = t_publish = 0.0
                timing_updates = 0

            log_writer.writerow([
                global_step, update_count, f"{pg_val:.6f}", f"{vf_val:.6f}",
                f"{ent_val:.4f}", f"{mean_rho:.4f}", f"{clip_frac:.4f}",
                sum(1 for s in segment_slots if s.ready), max_policy_lag,
                f"{elapsed:.1f}",
            ])
            log_file.flush()

    finally:
        # Shutdown actors.
        print("\nShutting down actors...")
        stop_event.set()
        # Mark all slots consumed so actors aren't blocked waiting.
        for slot in segment_slots:
            slot.mark_consumed()
        for p in actors:
            p.join(timeout=10)
            if p.is_alive():
                p.terminate()

    # Save final model (this is the only checkpoint — flywheel handles eval).
    final_path = run_dir / "final_model.pt"
    torch.save({
        "state_dict": learner_model.state_dict(),
        "model_kwargs": model_kwargs,
    }, final_path)
    torch.save({
        "state_dict": learner_model.state_dict(),
        "model_kwargs": model_kwargs,
        "optimizer": optimizer.state_dict(),
        "global_step": global_step,
    }, run_dir / "checkpoint.pt")

    log_file.close()

    elapsed = time.time() - start_time
    print(f"\n{stage_label} complete in {elapsed:.0f}s. Final model: {final_path}")

    return final_path


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    # Must set start method before any other multiprocessing usage.
    mp.set_start_method("spawn", force=True)

    args = parse_args()

    # Imports that need scripts/ on path.
    scripts_dir = str(Path(__file__).resolve().parent)
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    from model import DeckModel
    from strategy import PHASE_NAMES
    import decker_pyenv as decker

    COMBAT_PHASES = {PHASE_NAMES.index("combat")}
    CURATION_PHASES = {PHASE_NAMES.index(p) for p in ("reward", "deck_swap", "deck_rebuild")}

    # Validate.
    if (args.curation_only or args.shared) and not args.init_model:
        print("ERROR: --curation-only/--shared requires --init-model.",
              file=sys.stderr)
        sys.exit(1)

    # Device.
    if args.device == "auto":
        train_device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    else:
        train_device = torch.device(args.device)
    if train_device.type == "cuda":
        print(f"Learner on GPU ({torch.cuda.get_device_name(0)}), actors on CPU")
    else:
        print("Using CPU (all processes)")

    # Seed.
    if args.seed is None:
        args.seed = int.from_bytes(os.urandom(4), "big")
    print(f"Seed: {args.seed}")
    torch.manual_seed(args.seed)
    np.random.seed(args.seed)

    synergy_group = getattr(args, 'synergy_group', None)
    if synergy_group:
        owner = decker.subclass_for_synergy_group(synergy_group)
        if owner is None:
            print(f"ERROR: unknown synergy group '{synergy_group}'", file=sys.stderr)
            sys.exit(1)
        if args.subclass is None:
            args.subclass = owner
        elif args.subclass != owner:
            print(f"ERROR: synergy group '{synergy_group}' belongs to '{owner}', "
                  f"not '{args.subclass}'", file=sys.stderr)
            sys.exit(1)
        print(f"Synergy filter: {synergy_group} (subclass: {args.subclass})")
    if args.subclass is None:
        print("ERROR: --subclass is required", file=sys.stderr)
        sys.exit(1)

    # Feature dims.
    vec_tmp = decker.VecGauntletEnv(args.subclass, [0], args.race,
                                     synergy_filter=synergy_group)
    dims = vec_tmp.instance_dims()
    del vec_tmp
    global_dim, card_feat_dim, enemy_feat_dim, action_feat_dim, vocab_size = dims[:5]

    model_kwargs = dict(
        global_dim=global_dim,
        card_feat_dim=card_feat_dim,
        enemy_feat_dim=enemy_feat_dim,
        action_feat_dim=action_feat_dim,
        vocab_size=vocab_size,
    )

    # Run directory.
    run_id = uuid.uuid4().hex[:8]
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    parent_dir = Path(args.checkpoint_dir) / f"{args.subclass}_{timestamp}_{run_id}"

    seed_rng = np.random.RandomState(args.seed)

    deadline = (time.time() + args.max_time) if args.max_time else None
    if deadline:
        print(f"Wall-clock deadline: {args.max_time:.0f}s from now")

    best_combat_path = None
    best_picks_path = None

    if args.shared:
        # Shared mode: train on all phases simultaneously.
        best_combat_path = run_impala_training(
            dims, args,
            train_phase_ids=COMBAT_PHASES | CURATION_PHASES,
            reward_mode="fights-won",
            model_non_trainable=False,
            total_timesteps=args.shared_timesteps,
            stage_label="shared",
            train_device=train_device,
            run_dir=parent_dir,
            seed_rng=seed_rng,
            model_kwargs=model_kwargs,
            deadline=deadline,
        )
    else:
        run_combat = not args.curation_only
        run_picks = not args.combat_only

        # Stage 1: Combat.
        if run_combat:
            combat_dir = parent_dir / "combat" if run_picks else parent_dir
            best_combat_path = run_impala_training(
                dims, args,
                train_phase_ids=COMBAT_PHASES,
                reward_mode=args.reward_mode,
                model_non_trainable=False,
                total_timesteps=args.combat_timesteps,
                stage_label="combat",
                train_device=train_device,
                run_dir=combat_dir,
                seed_rng=seed_rng,
                model_kwargs=model_kwargs,
                deadline=deadline,
            )

        # Stage 2: Curation (reward picks, deck swaps, rebuilds).
        if run_picks:
            if deadline is not None and time.time() > deadline:
                print("[max-time] Skipping curation stage — deadline reached.")
            else:
                if best_combat_path:
                    args.init_model = str(best_combat_path)
                picks_dir = parent_dir / "picks" if run_combat else parent_dir
                best_picks_path = run_impala_training(
                    dims, args,
                    train_phase_ids=CURATION_PHASES,
                    reward_mode="fights-won",
                    model_non_trainable=True,
                    total_timesteps=args.curation_timesteps,
                    stage_label="curation",
                    train_device=train_device,
                    run_dir=picks_dir,
                    seed_rng=seed_rng,
                    model_kwargs=model_kwargs,
                    deadline=deadline,
                )

    # Summary.
    print("\n" + "=" * 60)
    print("Training complete.")
    if best_combat_path:
        print(f"  Combat model: {best_combat_path}")
    if best_picks_path:
        print(f"  Final model:  {best_picks_path}")

    # Write output manifest at checkpoint_dir level (not the nested
    # subdirectory). Flywheel mounts checkpoint_dir as /output, so the
    # manifest must be at /output/output_manifest.json with paths
    # relative to /output.
    checkpoint_root = Path(args.checkpoint_dir)
    artifacts = []
    if best_combat_path and best_combat_path.exists():
        artifacts.append({
            "type": "checkpoint",
            "path": str(best_combat_path.relative_to(checkpoint_root)),
            "stage": "combat",
            "metadata": {"subclass": args.subclass, "seed": args.seed},
        })
    if best_picks_path and best_picks_path.exists():
        artifacts.append({
            "type": "checkpoint",
            "path": str(best_picks_path.relative_to(checkpoint_root)),
            "stage": "curation",
            "metadata": {"subclass": args.subclass, "seed": args.seed},
        })
    manifest = {"artifacts": artifacts}
    manifest_path = checkpoint_root / "output_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2))
    print(f"  Manifest: {manifest_path}")
    write_flywheel_termination("normal")


if __name__ == "__main__":
    main()
