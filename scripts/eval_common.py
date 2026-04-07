"""Shared RL evaluation logic.

Runs a model through greedy game episodes and returns statistics.
Used by both train_impala.py (inline eval) and eval_checkpoint.py
(containerized eval). This is the single source of truth for how
RL models are evaluated.

Supports both sequential and parallel execution. Parallel mode
uses ProcessPoolExecutor to distribute episodes across workers.
"""

from __future__ import annotations

import os
import sys
import time
from concurrent.futures import ProcessPoolExecutor, TimeoutError as FuturesTimeout
from pathlib import Path

import numpy as np
import torch

# Ensure scripts/ is importable.
_scripts_dir = str(Path(__file__).resolve().parent)
if _scripts_dir not in sys.path:
    sys.path.insert(0, _scripts_dir)


# ---------------------------------------------------------------------------
# Worker state for parallel execution
# ---------------------------------------------------------------------------

_worker_model = None
_worker_subclass = None
_worker_race = None
_worker_synergy_group = None
_worker_dims = None


def _init_worker(
    checkpoint_path, model_kwargs, subclass, dims,
    race, synergy_group,
):
    """Initialize a worker process with a loaded model."""
    global _worker_model, _worker_subclass, _worker_race
    global _worker_synergy_group, _worker_dims

    from model import DeckModel

    _worker_subclass = subclass
    _worker_race = race
    _worker_synergy_group = synergy_group
    _worker_dims = dims

    _worker_model = DeckModel(**model_kwargs)
    state_dict, _ = load_checkpoint(checkpoint_path)
    _worker_model.load_state_dict(state_dict)
    _worker_model.eval()


def _run_episode(seed):
    """Run a single greedy episode in a worker process."""
    return _run_single_episode(
        _worker_model, _worker_subclass, seed,
        _worker_dims, _worker_race, _worker_synergy_group,
    )


# ---------------------------------------------------------------------------
# Core episode runner
# ---------------------------------------------------------------------------

def _run_single_episode(model, subclass, seed, dims, race, synergy_group):
    """Run one greedy episode. Returns (fights_won, steps)."""
    import decker_pyenv as decker

    _, card_feat_dim, enemy_feat_dim, action_feat_dim, \
        _, max_hand, max_enemies, max_actions = dims

    env = decker.GauntletEnv(
        subclass, seed, race, synergy_filter=synergy_group)
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
        obs_t = {}
        for k, v in obs.items():
            dtype = (
                torch.float32
                if v.dtype != np.int32
                else torch.int32
            )
            obs_t[k] = torch.as_tensor(
                v, dtype=dtype).unsqueeze(0)
        with torch.no_grad():
            scores, _ = model(obs_t)
        action = scores.argmax(dim=-1).item()
        n_legal = int(obs["n_actions"])
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

def evaluate_model(
    model, subclass, n_episodes, dims, device=None,
    seed_rng=None, seed=None, race="human",
    synergy_group=None,
):
    """Run greedy evaluation episodes sequentially.

    Parameters
    ----------
    model : DeckModel
        Model with forward() returning (scores, value).
    subclass : str
        Game subclass (e.g., 'dueling').
    n_episodes : int
        Number of episodes to run.
    dims : tuple
        From ``VecGauntletEnv.instance_dims()``.
    device : torch.device or None
        Device for inference (default: cpu).
    seed_rng : numpy.random.RandomState or None
        RNG for episode seeds.
    seed : int or None
        Integer seed (creates a RandomState internally).
    race : str
        Character race.
    synergy_group : str or None
        Optional synergy filter.

    Returns
    -------
    dict
        Stats with mean, median, std, min, max, p25, p75,
        episodes, fights_won.
    """
    if seed_rng is None:
        seed_rng = np.random.RandomState(
            seed if seed is not None else 42)

    was_training = model.training
    model.eval()
    fights = []

    for _ in range(n_episodes):
        ep_seed = int(seed_rng.randint(0, 2**31))
        won, _ = _run_single_episode(
            model, subclass, ep_seed, dims,
            race, synergy_group,
        )
        fights.append(won)

    if was_training:
        model.train()

    return _build_stats(fights, n_episodes)


def evaluate_parallel(
    checkpoint_path, model_kwargs, subclass, n_episodes,
    dims, seed=42, race="human", synergy_group=None,
    n_workers=None, timeout=300,
):
    """Run greedy evaluation episodes in parallel.

    Uses ProcessPoolExecutor to distribute episodes across worker
    processes. Each worker loads its own copy of the model from
    the checkpoint file.

    Parameters
    ----------
    checkpoint_path : str or Path
        Path to the checkpoint file.
    model_kwargs : dict
        Constructor kwargs for DeckModel.
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
        initializer=_init_worker,
        initargs=(
            str(checkpoint_path), model_kwargs, subclass,
            dims, race, synergy_group,
        ),
    ) as executor:
        futures = {
            executor.submit(_run_episode, s): s
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
    stats = _build_stats(fights, n_episodes)
    stats["total_steps"] = total_steps
    stats["elapsed_s"] = round(elapsed, 1)
    stats["timed_out"] = timed_out
    return stats


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _build_stats(fights, n_episodes):
    """Build stats dict from a list of fights won."""
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
    }


def _pad_obs(
    gf, hf_raw, ef_raw, af_raw,
    max_hand, card_feat_dim,
    max_enemies, enemy_feat_dim,
    max_actions, action_feat_dim,
):
    """Pad raw observation arrays to fixed sizes for model input."""
    hand = np.zeros(
        (max_hand, card_feat_dim), dtype=np.float32)
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
        "global": gf,
        "hand": hand,
        "enemies": enemies,
        "action_feats": action_feats,
        "action_mask": action_mask,
        "n_hand": np.int32(len(hf_raw)),
        "n_enemies": np.int32(len(ef_raw)),
        "n_actions": np.int32(len(af_raw)),
    }


def resolve_dims(subclass, race="human", synergy_group=None):
    """Resolve model dimensions from the game engine.

    Returns the full dims tuple from
    ``VecGauntletEnv.instance_dims()``.
    """
    import decker_pyenv as decker

    vec_tmp = decker.VecGauntletEnv(
        subclass, [0], race,
        synergy_filter=synergy_group)
    dims = vec_tmp.instance_dims()
    del vec_tmp
    return dims


def load_checkpoint(path):
    """Load a checkpoint and return (state_dict, model_kwargs).

    Supports both self-describing checkpoints (dict with
    ``state_dict`` and ``model_kwargs`` keys) and legacy
    checkpoints (bare state_dict).
    """
    ckpt = torch.load(
        path, map_location="cpu", weights_only=False)
    if isinstance(ckpt, dict) and "state_dict" in ckpt:
        return ckpt["state_dict"], ckpt.get("model_kwargs")
    return ckpt, None
