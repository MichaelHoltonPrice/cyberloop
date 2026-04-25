"""Baseline Cyberloop bot for the ImproveBot pattern."""


def player_fn(env, obs_json, action_labels):
    """Pick the first legal action."""
    del env, obs_json
    return 0 if action_labels else 0
