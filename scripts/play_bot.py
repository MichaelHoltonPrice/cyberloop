"""Game-playing harness for bot evaluation.

A "bot" is any Python module defining a callable with the signature:

    player_fn(env, obs_json: str, action_labels: list[str]) -> int

Usage (as library):
    from play_bot import play_parallel

    # Evaluate a custom bot
    stats = play_parallel(bot_path="path/to/bot.py", subclass="dueling", episodes=4000)

    # Random baseline
    stats = play_parallel(bot_path=None, subclass="dueling", episodes=4000)
"""

import os
import sys
import time
from concurrent.futures import ProcessPoolExecutor, TimeoutError as FuturesTimeout
from pathlib import Path

import numpy as np

# Ensure scripts/ is importable.
_scripts_dir = str(Path(__file__).resolve().parent)
if _scripts_dir not in sys.path:
    sys.path.insert(0, _scripts_dir)


# Default per-episode timeout (seconds).
DEFAULT_EPISODE_TIMEOUT = 60

# Max steps per episode (catches non-terminating bots).
MAX_STEPS_PER_EPISODE = 500_000

# Speed screen: run a small sample before the full eval.
# If the sample is too slow, abort early.
SPEED_SCREEN_EPISODES = 10
SPEED_SCREEN_THRESHOLD = 0.5  # seconds per episode

# Abort if this many episodes timeout consecutively.
CONSECUTIVE_TIMEOUT_LIMIT = 3


def play_episode(player_fn, subclass, seed, race="human", background="soldier"):
    """Run one episode. Returns (fights_won, steps)."""
    import decker_pyenv as decker

    env = decker.GauntletEnv(subclass, seed, race, background=background)
    steps = 0

    while not env.is_done and steps < MAX_STEPS_PER_EPISODE:
        obs_json = env.observe()
        labels = env.legal_action_labels()
        if not labels:
            break

        action = player_fn(env, obs_json, labels)

        if action < 0 or action >= len(labels):
            action = 0

        env.step(action)
        steps += 1

    return env.fights_won, steps


# ---------------------------------------------------------------------------
# Worker for parallel execution
# ---------------------------------------------------------------------------

_worker_player_fn = None
_worker_subclass = None
_worker_race = None
_worker_background = None


def _init_worker(bot_path, subclass, race, background):
    """Initialize the worker process with a loaded player_fn."""
    global _worker_player_fn, _worker_subclass, _worker_race, _worker_background
    _worker_subclass = subclass
    _worker_race = race
    _worker_background = background

    if bot_path is not None:
        import importlib.util
        spec = importlib.util.spec_from_file_location("custom_bot", bot_path)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        _worker_player_fn = mod.player_fn
    else:
        import random
        _worker_player_fn = lambda env, obs_json, labels: random.randrange(len(labels))


def _run_episode(seed):
    """Run a single episode in a worker process."""
    won, steps = play_episode(
        _worker_player_fn, _worker_subclass, seed,
        race=_worker_race, background=_worker_background,
    )
    return won, steps


# ---------------------------------------------------------------------------
# Main API
# ---------------------------------------------------------------------------

def play_parallel(bot_path, subclass, episodes, seed=42, race="human",
                  background="soldier", verbose=False, n_workers=None,
                  timeout=DEFAULT_EPISODE_TIMEOUT,
                  speed_screen_episodes=SPEED_SCREEN_EPISODES,
                  speed_screen_threshold=SPEED_SCREEN_THRESHOLD):
    """Run N episodes in parallel and return stats dict.

    Args:
        bot_path: Path to bot .py file (must define player_fn).
                  Pass None for the random baseline.
        subclass: Fighter sub-class (dueling, two_handed, defense).
        episodes: Number of episodes to run.
        seed: Base RNG seed.
        race: Character race.
        background: Character background.
        verbose: Print progress.
        n_workers: Number of parallel workers (default: CPU count).
        timeout: Per-episode timeout in seconds.
        speed_screen_episodes: Number of episodes for speed screen.
            Set to 0 to disable.
        speed_screen_threshold: Max avg seconds per episode before
            aborting as too slow.

    Returns:
        Stats dict with keys: fights_won, mean, median, std, min, max,
        p25, p75, episodes, total_steps, elapsed_s, timed_out, error_count.
    """
    if n_workers is None:
        n_workers = min(os.cpu_count() or 4, episodes)

    # Speed screen: run a small sample sequentially to check for
    # slow bots before committing to the full parallel eval.
    if (speed_screen_episodes > 0
            and episodes > speed_screen_episodes
            and bot_path is not None):
        screen_result = _run_speed_screen(
            bot_path, subclass, seed, race, background,
            speed_screen_episodes, speed_screen_threshold,
            timeout, verbose)
        if screen_result is not None:
            return screen_result

    # Full parallel eval.
    seeds = [seed + i * 1000 for i in range(episodes)]
    fights = []
    total_steps = 0
    timed_out = 0
    consecutive_timeouts = 0
    aborted = False
    error_types = {}
    t0 = time.time()

    executor = ProcessPoolExecutor(
        max_workers=n_workers,
        initializer=_init_worker,
        initargs=(bot_path, subclass, race, background),
    )
    try:
        futures = {executor.submit(_run_episode, s): s for s in seeds}
        done_count = 0
        for future in futures:
            try:
                won, steps = future.result(timeout=timeout)
                fights.append(won)
                total_steps += steps
                consecutive_timeouts = 0
            except FuturesTimeout:
                fights.append(0)
                timed_out += 1
                consecutive_timeouts += 1
                if consecutive_timeouts >= CONSECUTIVE_TIMEOUT_LIMIT:
                    if verbose:
                        print(
                            f"  [abort] {CONSECUTIVE_TIMEOUT_LIMIT}"
                            f" consecutive timeouts — aborting")
                    aborted = True
                    break
            except Exception as e:
                if verbose:
                    print(f"  ERROR in episode: {e}")
                fights.append(0)
                consecutive_timeouts = 0
                type_name = type(e).__name__
                if type_name not in error_types:
                    error_types[type_name] = {
                        "count": 0, "example": str(e)}
                error_types[type_name]["count"] += 1
            done_count += 1
            if verbose and done_count % max(1, episodes // 10) == 0:
                elapsed = time.time() - t0
                print(f"  [{done_count}/{episodes}] mean={np.mean(fights):.1f} "
                      f"({elapsed:.1f}s)")
    finally:
        executor.shutdown(wait=False, cancel_futures=True)

    elapsed = time.time() - t0
    error_count = sum(e["count"] for e in error_types.values())
    return _build_stats(fights, total_steps, elapsed, timed_out,
                        error_count, episodes, aborted, error_types)


def _build_stats(fights, total_steps, elapsed, timed_out,
                 error_count, episodes, aborted=False, error_types=None):
    """Build the stats dict from episode results."""
    fights_arr = np.array(fights) if fights else np.array([0])
    result = {
        "fights_won": [int(x) for x in fights],
        "mean": round(float(fights_arr.mean()), 1),
        "median": round(float(np.median(fights_arr)), 1),
        "std": round(float(fights_arr.std()), 1),
        "min": int(fights_arr.min()),
        "max": int(fights_arr.max()),
        "p25": round(float(np.percentile(fights_arr, 25)), 1),
        "p75": round(float(np.percentile(fights_arr, 75)), 1),
        "episodes": episodes,
        "total_steps": total_steps,
        "elapsed_s": round(elapsed, 1),
        "timed_out": timed_out,
        "error_count": error_count,
    }
    if error_types:
        result["error_types"] = [
            {"type": t, **info}
            for t, info in error_types.items()
        ]
    if aborted:
        result["aborted"] = True
        result["abort_reason"] = "consecutive_timeouts"
        result["abort_message"] = (
            f"Aborted after {CONSECUTIVE_TIMEOUT_LIMIT} consecutive"
            f" episode timeouts. Your bot is too slow or has an"
            f" infinite loop. Fix performance before retrying.")
    return result


def _run_speed_screen(bot_path, subclass, seed, race, background,
                       n_episodes, threshold, timeout, verbose):
    """Run a small sample to check bot speed.

    Returns a result dict if the bot is too slow (caller should
    return it immediately). Returns None if the bot passes.
    """
    if verbose:
        print(f"  [speed-screen] Running {n_episodes} episodes...")

    t0 = time.time()
    fights = []
    total_steps = 0

    # Run sequentially with a single worker to get clean timing.
    executor = ProcessPoolExecutor(
        max_workers=1,
        initializer=_init_worker,
        initargs=(bot_path, subclass, race, background),
    )
    try:
        for i in range(n_episodes):
            ep_seed = seed + i * 1000
            future = executor.submit(_run_episode, ep_seed)
            try:
                won, steps = future.result(timeout=timeout)
                fights.append(won)
                total_steps += steps
            except FuturesTimeout:
                elapsed = time.time() - t0
                if verbose:
                    print(
                        f"  [speed-screen] Episode {i+1}"
                        f" timed out — bot too slow")
                return {
                    "fights_won": [],
                    "mean": 0.0, "median": 0.0, "std": 0.0,
                    "min": 0, "max": 0,
                    "p25": 0.0, "p75": 0.0,
                    "episodes": n_episodes,
                    "total_steps": total_steps,
                    "elapsed_s": round(elapsed, 1),
                    "timed_out": 1,
                    "error_count": 0,
                    "aborted": True,
                    "abort_reason": "too_slow",
                    "abort_message": (
                        f"Episode timed out during speed screen"
                        f" ({timeout}s limit). Bot is too slow"
                        f" for full evaluation. Fix performance"
                        f" before retrying."),
                }
            except Exception:
                fights.append(0)
    finally:
        executor.shutdown(wait=False, cancel_futures=True)

    elapsed = time.time() - t0
    avg_s = elapsed / max(n_episodes, 1)

    if avg_s > threshold:
        if verbose:
            print(
                f"  [speed-screen] Too slow: {avg_s:.2f}s/episode"
                f" (threshold: {threshold}s)")
        fights_arr = np.array(fights) if fights else np.array([0])
        return {
            "fights_won": [int(x) for x in fights],
            "mean": round(float(fights_arr.mean()), 1),
            "median": round(float(np.median(fights_arr)), 1),
            "std": round(float(fights_arr.std()), 1),
            "min": int(fights_arr.min()),
            "max": int(fights_arr.max()),
            "p25": round(float(np.percentile(fights_arr, 25)), 1),
            "p75": round(float(np.percentile(fights_arr, 75)), 1),
            "episodes": n_episodes,
            "total_steps": total_steps,
            "elapsed_s": round(elapsed, 1),
            "timed_out": 0,
            "error_count": 0,
            "aborted": True,
            "abort_reason": "too_slow",
            "avg_s_per_episode": round(avg_s, 3),
            "threshold_s_per_episode": threshold,
            "abort_message": (
                f"Bot averaged {avg_s:.2f}s/episode in speed"
                f" screen (threshold: {threshold}s). Too slow"
                f" for full evaluation. Fix performance before"
                f" retrying."),
        }

    if verbose:
        print(
            f"  [speed-screen] Passed: {avg_s:.3f}s/episode"
            f" (threshold: {threshold}s)")
    return None
