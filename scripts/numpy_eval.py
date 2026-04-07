"""Numpy-only RL evaluation — no PyTorch dependency.

Provides NumpyModel (a pure-numpy reimplementation of DeckModel's
forward pass) and evaluate_parallel() for running greedy episodes
across multiple worker processes.

~5x faster than PyTorch for single-batch CPU inference because it
avoids PyTorch's per-call dispatch overhead, tensor creation, and
Python→C++ round-trips. The model is small enough (~302K params)
that numpy matrix multiplies dominate.
"""

from __future__ import annotations

import os
import sys
import time
from concurrent.futures import ProcessPoolExecutor, TimeoutError as FuturesTimeout
from pathlib import Path

import numpy as np

_scripts_dir = str(Path(__file__).resolve().parent)
if _scripts_dir not in sys.path:
    sys.path.insert(0, _scripts_dir)

PHASE_OFFSET = 8
PHASE_COUNT = 7
COMBAT_PHASE_INDEX = 0


# ---------------------------------------------------------------------------
# Numpy model
# ---------------------------------------------------------------------------

class NumpyModel:
    """Pure-numpy reimplementation of DeckModel.forward() for inference.

    Loads weights from a PyTorch state_dict (converted to numpy arrays)
    and implements the same forward pass: global encoder, card encoder
    with embedding + masked mean pool, enemy encoder with masked mean
    pool, and per-action scoring with combat/noncombat heads.
    """

    def __init__(self, state_dict: dict, vocab_size: int):
        self.vocab_size = vocab_size

        def w(key):
            v = state_dict[key]
            return v.numpy() if hasattr(v, 'numpy') else np.array(v)

        # Global encoder: Linear(global_dim, 128) -> ReLU -> Linear(128, 128) -> ReLU
        self.g_w1 = w('global_enc.0.weight')
        self.g_b1 = w('global_enc.0.bias')
        self.g_w2 = w('global_enc.2.weight')
        self.g_b2 = w('global_enc.2.bias')

        # Card encoder: Embedding(vocab_size, 16) + Linear(cont+16, 128) -> ReLU
        self.card_embed_w = w('card_embed.weight')
        self.c_w = w('card_enc.0.weight')
        self.c_b = w('card_enc.0.bias')

        # Enemy encoder: Linear(enemy_dim, 128) -> ReLU
        self.e_w = w('enemy_enc.0.weight')
        self.e_b = w('enemy_enc.0.bias')

        # Combat action scorer: Linear(action_dim+384, 128) -> ReLU -> Linear(128, 1)
        self.ca_w1 = w('combat_action_scorer.0.weight')
        self.ca_b1 = w('combat_action_scorer.0.bias')
        self.ca_w2 = w('combat_action_scorer.2.weight')
        self.ca_b2 = w('combat_action_scorer.2.bias')

        # Noncombat action scorer
        self.na_w1 = w('noncombat_action_scorer.0.weight')
        self.na_b1 = w('noncombat_action_scorer.0.bias')
        self.na_w2 = w('noncombat_action_scorer.2.weight')
        self.na_b2 = w('noncombat_action_scorer.2.bias')

        # Value heads: Linear(384, 128) -> ReLU -> Linear(128, 1)
        self.cv_w1 = w('combat_value_head.0.weight')
        self.cv_b1 = w('combat_value_head.0.bias')
        self.cv_w2 = w('combat_value_head.2.weight')
        self.cv_b2 = w('combat_value_head.2.bias')

        self.nv_w1 = w('noncombat_value_head.0.weight')
        self.nv_b1 = w('noncombat_value_head.0.bias')
        self.nv_w2 = w('noncombat_value_head.2.weight')
        self.nv_b2 = w('noncombat_value_head.2.bias')

    def load_state_dict(self, state_dict: dict):
        """Update weights from a state_dict (numpy arrays)."""
        def w(key):
            v = state_dict[key]
            return v.numpy() if hasattr(v, 'numpy') else np.array(v)

        self.g_w1 = w('global_enc.0.weight'); self.g_b1 = w('global_enc.0.bias')
        self.g_w2 = w('global_enc.2.weight'); self.g_b2 = w('global_enc.2.bias')
        self.card_embed_w = w('card_embed.weight')
        self.c_w = w('card_enc.0.weight'); self.c_b = w('card_enc.0.bias')
        self.e_w = w('enemy_enc.0.weight'); self.e_b = w('enemy_enc.0.bias')
        self.ca_w1 = w('combat_action_scorer.0.weight'); self.ca_b1 = w('combat_action_scorer.0.bias')
        self.ca_w2 = w('combat_action_scorer.2.weight'); self.ca_b2 = w('combat_action_scorer.2.bias')
        self.na_w1 = w('noncombat_action_scorer.0.weight'); self.na_b1 = w('noncombat_action_scorer.0.bias')
        self.na_w2 = w('noncombat_action_scorer.2.weight'); self.na_b2 = w('noncombat_action_scorer.2.bias')
        self.cv_w1 = w('combat_value_head.0.weight'); self.cv_b1 = w('combat_value_head.0.bias')
        self.cv_w2 = w('combat_value_head.2.weight'); self.cv_b2 = w('combat_value_head.2.bias')
        self.nv_w1 = w('noncombat_value_head.0.weight'); self.nv_b1 = w('noncombat_value_head.0.bias')
        self.nv_w2 = w('noncombat_value_head.2.weight'); self.nv_b2 = w('noncombat_value_head.2.bias')

    def forward(self, obs: dict) -> int:
        """Return the greedy action index for a single observation."""
        gf = obs['global']
        hand = obs['hand']
        enemies = obs['enemies']
        af = obs['action_feats']
        n_hand = int(obs['n_hand'])
        n_enemies = int(obs['n_enemies'])
        n_actions = int(obs['n_actions'])

        # Global encoder
        g = np.maximum(0, gf @ self.g_w1.T + self.g_b1)
        g = np.maximum(0, g @ self.g_w2.T + self.g_b2)

        # Card encoder + masked mean pool
        if n_hand > 0:
            cont = hand[:n_hand, :-self.vocab_size]
            onehot = hand[:n_hand, -self.vocab_size:]
            vocab_idx = onehot.argmax(axis=-1).clip(
                0, self.card_embed_w.shape[0] - 1)
            embed = self.card_embed_w[vocab_idx]
            card_h = np.maximum(0,
                np.concatenate([cont, embed], axis=-1) @ self.c_w.T
                + self.c_b)
            card_pool = card_h.mean(axis=0)
        else:
            card_pool = np.zeros(128, dtype=np.float32)

        # Enemy encoder + masked mean pool
        if n_enemies > 0:
            enemy_h = np.maximum(0,
                enemies[:n_enemies] @ self.e_w.T + self.e_b)
            enemy_pool = enemy_h.mean(axis=0)
        else:
            enemy_pool = np.zeros(128, dtype=np.float32)

        # Phase detection
        is_combat = (
            gf[PHASE_OFFSET:PHASE_OFFSET + PHASE_COUNT].argmax()
            == COMBAT_PHASE_INDEX)

        # State context
        ctx = np.concatenate([g, card_pool, enemy_pool])

        # Score actions
        if n_actions > 0:
            ctx_exp = np.broadcast_to(ctx, (n_actions, 384))
            combined = np.concatenate(
                [af[:n_actions], ctx_exp], axis=-1)
            if is_combat:
                h = np.maximum(0,
                    combined @ self.ca_w1.T + self.ca_b1)
                scores = (h @ self.ca_w2.T + self.ca_b2).ravel()
            else:
                h = np.maximum(0,
                    combined @ self.na_w1.T + self.na_b1)
                scores = (h @ self.na_w2.T + self.na_b2).ravel()
            return int(scores.argmax())
        return 0

    def get_action_and_value_batch(self, obs_batch: dict):
        """Batched inference for training actors.

        Takes a dict of numpy arrays with leading batch dimension (B, ...)
        matching the format from VecGauntletEnv.get_obs_all() /
        np_to_obs_batch().

        Returns:
            actions: (B,) int32 — sampled actions
            log_probs: (B,) float32 — log probability of sampled action
            values: (B,) float32 — state value estimate
        """
        gf_batch = obs_batch['global']            # (B, global_dim)
        hand_batch = obs_batch['hand']             # (B, max_hand, card_feat_dim)
        enemies_batch = obs_batch['enemies']       # (B, max_enemies, enemy_feat_dim)
        af_batch = obs_batch['action_feats']       # (B, max_actions, action_feat_dim)
        amask_batch = obs_batch['action_mask']     # (B, max_actions)
        nh_batch = obs_batch['n_hand']             # (B,)
        ne_batch = obs_batch['n_enemies']          # (B,)
        na_batch = obs_batch['n_actions']           # (B,)

        B = gf_batch.shape[0]
        max_actions = af_batch.shape[1]
        hidden = 128

        actions = np.zeros(B, dtype=np.int32)
        log_probs = np.zeros(B, dtype=np.float32)
        values = np.zeros(B, dtype=np.float32)

        for i in range(B):
            gf = gf_batch[i]
            n_hand = int(nh_batch[i])
            n_enemies = int(ne_batch[i])
            n_actions = int(na_batch[i])

            # Global encoder
            g = np.maximum(0, gf @ self.g_w1.T + self.g_b1)
            g = np.maximum(0, g @ self.g_w2.T + self.g_b2)

            # Card encoder + masked mean pool
            if n_hand > 0:
                hand = hand_batch[i, :n_hand]
                cont = hand[:, :-self.vocab_size]
                onehot = hand[:, -self.vocab_size:]
                vocab_idx = onehot.argmax(axis=-1).clip(
                    0, self.card_embed_w.shape[0] - 1)
                embed = self.card_embed_w[vocab_idx]
                card_h = np.maximum(0,
                    np.concatenate([cont, embed], axis=-1) @ self.c_w.T
                    + self.c_b)
                card_pool = card_h.mean(axis=0)
            else:
                card_pool = np.zeros(hidden, dtype=np.float32)

            # Enemy encoder + masked mean pool
            if n_enemies > 0:
                enemy_h = np.maximum(0,
                    enemies_batch[i, :n_enemies] @ self.e_w.T + self.e_b)
                enemy_pool = enemy_h.mean(axis=0)
            else:
                enemy_pool = np.zeros(hidden, dtype=np.float32)

            # Phase + context
            is_combat = (
                gf[PHASE_OFFSET:PHASE_OFFSET + PHASE_COUNT].argmax()
                == COMBAT_PHASE_INDEX)
            ctx = np.concatenate([g, card_pool, enemy_pool])

            # Value
            if is_combat:
                vh = np.maximum(0, ctx @ self.cv_w1.T + self.cv_b1)
                values[i] = float((vh @ self.cv_w2.T + self.cv_b2).item())
            else:
                vh = np.maximum(0, ctx @ self.nv_w1.T + self.nv_b1)
                values[i] = float((vh @ self.nv_w2.T + self.nv_b2).item())

            # Score actions → softmax → sample
            if n_actions > 0:
                ctx_exp = np.broadcast_to(ctx, (n_actions, 384))
                combined = np.concatenate(
                    [af_batch[i, :n_actions], ctx_exp], axis=-1)
                if is_combat:
                    h = np.maximum(0, combined @ self.ca_w1.T + self.ca_b1)
                    scores = (h @ self.ca_w2.T + self.ca_b2).ravel()
                else:
                    h = np.maximum(0, combined @ self.na_w1.T + self.na_b1)
                    scores = (h @ self.na_w2.T + self.na_b2).ravel()

                # Stable softmax
                scores_shifted = scores - scores.max()
                exp_s = np.exp(scores_shifted)
                probs = exp_s / exp_s.sum()

                # Sample action
                action = int(np.random.choice(n_actions, p=probs))
                actions[i] = action
                log_probs[i] = float(np.log(probs[action] + 1e-8))
            else:
                actions[i] = 0
                log_probs[i] = 0.0

        return actions, log_probs, values

    def get_value_batch(self, obs_batch: dict):
        """Batched value computation (for bootstrap)."""
        gf_batch = obs_batch['global']
        hand_batch = obs_batch['hand']
        enemies_batch = obs_batch['enemies']
        nh_batch = obs_batch['n_hand']
        ne_batch = obs_batch['n_enemies']

        B = gf_batch.shape[0]
        hidden = 128
        values = np.zeros(B, dtype=np.float32)

        for i in range(B):
            gf = gf_batch[i]
            n_hand = int(nh_batch[i])
            n_enemies = int(ne_batch[i])

            g = np.maximum(0, gf @ self.g_w1.T + self.g_b1)
            g = np.maximum(0, g @ self.g_w2.T + self.g_b2)

            if n_hand > 0:
                hand = hand_batch[i, :n_hand]
                cont = hand[:, :-self.vocab_size]
                onehot = hand[:, -self.vocab_size:]
                vocab_idx = onehot.argmax(axis=-1).clip(
                    0, self.card_embed_w.shape[0] - 1)
                embed = self.card_embed_w[vocab_idx]
                card_h = np.maximum(0,
                    np.concatenate([cont, embed], axis=-1) @ self.c_w.T
                    + self.c_b)
                card_pool = card_h.mean(axis=0)
            else:
                card_pool = np.zeros(hidden, dtype=np.float32)

            if n_enemies > 0:
                enemy_h = np.maximum(0,
                    enemies_batch[i, :n_enemies] @ self.e_w.T + self.e_b)
                enemy_pool = enemy_h.mean(axis=0)
            else:
                enemy_pool = np.zeros(hidden, dtype=np.float32)

            is_combat = (
                gf[PHASE_OFFSET:PHASE_OFFSET + PHASE_COUNT].argmax()
                == COMBAT_PHASE_INDEX)
            ctx = np.concatenate([g, card_pool, enemy_pool])

            if is_combat:
                vh = np.maximum(0, ctx @ self.cv_w1.T + self.cv_b1)
                values[i] = float((vh @ self.cv_w2.T + self.cv_b2).item())
            else:
                vh = np.maximum(0, ctx @ self.nv_w1.T + self.nv_b1)
                values[i] = float((vh @ self.nv_w2.T + self.nv_b2).item())

        return values


# ---------------------------------------------------------------------------
# Observation padding (same as eval_common._pad_obs)
# ---------------------------------------------------------------------------

def _pad_obs(gf, hf_raw, ef_raw, af_raw,
             max_hand, card_feat_dim,
             max_enemies, enemy_feat_dim,
             max_actions, action_feat_dim):
    """Pad raw observation arrays to fixed sizes for model input."""
    hand = np.zeros((max_hand, card_feat_dim), dtype=np.float32)
    for j, c in enumerate(hf_raw[:max_hand]):
        hand[j] = c
    enemies = np.zeros(
        (max_enemies, enemy_feat_dim), dtype=np.float32)
    for j, e in enumerate(ef_raw[:max_enemies]):
        enemies[j] = e
    action_feats = np.zeros(
        (max_actions, action_feat_dim), dtype=np.float32)
    action_mask = np.zeros(max_actions, dtype=np.float32)
    for j, a in enumerate(af_raw[:max_actions]):
        action_feats[j] = a
        action_mask[j] = 1.0
    return {
        'global': gf,
        'hand': hand,
        'enemies': enemies,
        'action_feats': action_feats,
        'action_mask': action_mask,
        'n_hand': np.int32(len(hf_raw)),
        'n_enemies': np.int32(len(ef_raw)),
        'n_actions': np.int32(len(af_raw)),
    }


# ---------------------------------------------------------------------------
# Checkpoint loading (torch-free: uses numpy .npz or torch with lazy import)
# ---------------------------------------------------------------------------

def load_checkpoint_numpy(path):
    """Load a checkpoint and return (state_dict_numpy, model_kwargs).

    Handles both .npz (pre-converted) and .pt (lazy torch import).
    """
    path = str(path)
    if path.endswith('.npz'):
        data = np.load(path, allow_pickle=True)
        state_dict = dict(data['state_dict'].item())
        model_kwargs = data.get('model_kwargs')
        if model_kwargs is not None:
            model_kwargs = model_kwargs.item()
        return state_dict, model_kwargs

    # Fall back to torch for .pt files.
    import torch
    ckpt = torch.load(path, map_location='cpu', weights_only=False)
    if isinstance(ckpt, dict) and 'state_dict' in ckpt:
        sd = {k: v.numpy() for k, v in ckpt['state_dict'].items()}
        return sd, ckpt.get('model_kwargs')
    sd = {k: v.numpy() for k, v in ckpt.items()}
    return sd, None


def resolve_dims(subclass, race='human', synergy_group=None):
    """Resolve model dimensions from the game engine."""
    import decker_pyenv as decker
    vec_tmp = decker.VecGauntletEnv(
        subclass, [0], race, synergy_filter=synergy_group)
    dims = vec_tmp.instance_dims()
    del vec_tmp
    return dims


# ---------------------------------------------------------------------------
# Worker state for parallel execution
# ---------------------------------------------------------------------------

_np_model = None
_np_subclass = None
_np_race = None
_np_synergy_group = None
_np_dims = None


def _np_init_worker(checkpoint_path, subclass, dims,
                    race, synergy_group):
    """Initialize a worker process with a NumpyModel."""
    global _np_model, _np_subclass, _np_dims
    global _np_race, _np_synergy_group
    _np_subclass = subclass
    _np_dims = dims
    _np_race = race
    _np_synergy_group = synergy_group
    sd, _ = load_checkpoint_numpy(checkpoint_path)
    _np_model = NumpyModel(sd, dims[4])


def _np_run_episode(seed):
    """Run a single greedy episode in a worker process."""
    import decker_pyenv as decker

    _, card_feat_dim, enemy_feat_dim, action_feat_dim, \
        _, max_hand, max_enemies, max_actions = _np_dims

    env = decker.GauntletEnv(
        _np_subclass, seed, _np_race,
        synergy_filter=_np_synergy_group)
    gf = np.array(env.get_global_features(), dtype=np.float32)
    hf_raw = env.get_hand_features()
    ef_raw = env.get_enemy_features()
    af_raw = env.get_action_features()

    obs = _pad_obs(
        gf, hf_raw, ef_raw, af_raw,
        max_hand, card_feat_dim,
        max_enemies, enemy_feat_dim,
        max_actions, action_feat_dim,
    )

    max_steps = 500_000
    max_steps_without_fight = 1000
    step_count = 0
    last_fights_won = env.fights_won
    steps_since_last_fight = 0

    while not env.is_done and step_count < max_steps:
        action = _np_model.forward(obs)
        n_legal = int(obs['n_actions'])
        if action >= n_legal:
            action = 0

        gf, hf_raw, ef_raw, af_raw, reward, done, _ = (
            env.step_features(action))
        step_count += 1

        current_fights = env.fights_won
        if current_fights > last_fights_won:
            last_fights_won = current_fights
            steps_since_last_fight = 0
        else:
            steps_since_last_fight += 1
        if steps_since_last_fight > max_steps_without_fight:
            break

        if not done:
            obs = _pad_obs(
                np.array(gf, dtype=np.float32),
                hf_raw, ef_raw, af_raw,
                max_hand, card_feat_dim,
                max_enemies, enemy_feat_dim,
                max_actions, action_feat_dim,
            )

    return env.fights_won, step_count


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def evaluate_parallel(
    checkpoint_path, subclass, n_episodes, dims,
    seed=42, race='human', synergy_group=None,
    n_workers=None, timeout=300,
):
    """Run greedy evaluation episodes in parallel using numpy inference.

    Parameters
    ----------
    checkpoint_path : str or Path
        Path to checkpoint file (.pt or .npz).
    subclass : str
        Game subclass.
    n_episodes : int
        Number of episodes.
    dims : tuple
        From ``VecGauntletEnv.instance_dims()``.
    seed : int
        Base seed for episode RNG.
    race : str
        Character race.
    synergy_group : str or None
        Optional synergy filter.
    n_workers : int or None
        Number of parallel workers (default: CPU count).
    timeout : int
        Per-episode timeout in seconds.

    Returns
    -------
    dict
        Stats with mean, median, std, min, max, p25, p75,
        episodes, fights_won, elapsed_s, timed_out.
    """
    if n_workers is None:
        n_workers = min(os.cpu_count() or 4, n_episodes)

    seeds = [seed + i * 1000 for i in range(n_episodes)]
    fights = []
    total_steps = 0
    timed_out = 0
    t0 = time.time()

    with ProcessPoolExecutor(
        max_workers=n_workers,
        initializer=_np_init_worker,
        initargs=(
            str(checkpoint_path), subclass, dims,
            race, synergy_group,
        ),
    ) as executor:
        futures = {
            executor.submit(_np_run_episode, s): s
            for s in seeds
        }
        for future in futures:
            try:
                won, steps = future.result(timeout=timeout)
                fights.append(won)
                total_steps += steps
            except FuturesTimeout:
                fights.append(0)
                timed_out += 1
            except Exception as e:
                print(f"  Episode error: {e}", file=sys.stderr)
                fights.append(0)

    elapsed = time.time() - t0
    fights_arr = np.array(fights)
    return {
        "mean": float(np.mean(fights_arr)),
        "median": float(np.median(fights_arr)),
        "std": float(np.std(fights_arr)),
        "min": int(np.min(fights_arr)),
        "max": int(np.max(fights_arr)),
        "p25": float(np.percentile(fights_arr, 25)),
        "p75": float(np.percentile(fights_arr, 75)),
        "episodes": int(n_episodes),
        "fights_won": fights_arr.tolist(),
        "total_steps": total_steps,
        "elapsed_s": round(elapsed, 1),
        "timed_out": timed_out,
    }
