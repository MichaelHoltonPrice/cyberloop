"""Composable decision strategies for the Decker gauntlet.

A Strategy decides a single action given game state. A CompositeStrategy
routes each decision to a per-phase strategy, allowing any combination of
RL model, heuristic, or scripted policies across decision types.

Phases (matching ObservedPhase order in global feature one-hot):
    0: combat          - play cards, end turn
    1: reward          - pick or skip a battle reward card
    2: collection_overflow - trim excess cards
    3: innate_choice   - choose an innate card
    4: deck_swap       - swap a new card into the play deck
    5: deck_rebuild    - rewrite the play deck at rest stops
    6: game_over       - terminal (no decisions)

Usage:
    from strategy import CompositeStrategy, ModelStrategy, AlwaysSkip

    # Model for everything, but always skip battle rewards:
    s = CompositeStrategy(
        default=ModelStrategy(model, device),
        reward=AlwaysSkip(),
    )

    # In an episode loop:
    action, trainable = s.choose(obs_dict, n_legal)
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Optional

import numpy as np
import torch

# Must match ObservedPhase order and PHASE_OFFSET/PHASE_COUNT in model.py.
PHASE_OFFSET = 8
PHASE_COUNT = 7
PHASE_NAMES = [
    "combat",
    "reward",
    "collection_overflow",
    "innate_choice",
    "deck_swap",
    "deck_rebuild",
    "game_over",
]


def phase_from_obs(obs: dict) -> int:
    """Extract phase index from an obs dict (single or batched)."""
    g = obs["global"]
    if g.ndim == 1:
        return int(g[PHASE_OFFSET: PHASE_OFFSET + PHASE_COUNT].argmax())
    return int(g[0, PHASE_OFFSET: PHASE_OFFSET + PHASE_COUNT].argmax())


def phase_name(phase_id: int) -> str:
    if 0 <= phase_id < len(PHASE_NAMES):
        return PHASE_NAMES[phase_id]
    return "unknown"


# ---------------------------------------------------------------------------
# Base class
# ---------------------------------------------------------------------------

class Strategy(ABC):
    """Picks an action index given observation state."""

    # Whether steps chosen by this strategy should generate training gradients.
    trainable: bool = False

    @abstractmethod
    def choose(self, obs: dict, n_legal: int) -> int:
        """Return an action index in [0, n_legal)."""
        ...


# ---------------------------------------------------------------------------
# Concrete strategies
# ---------------------------------------------------------------------------

class ModelStrategy(Strategy):
    """Use a trained RL model (greedy argmax)."""

    trainable = True

    def __init__(self, model, device=None):
        self.model = model
        self.device = device or torch.device("cpu")

    def choose(self, obs: dict, n_legal: int) -> int:
        obs_t = {}
        for k, v in obs.items():
            dtype = torch.float32 if v.dtype != np.int32 else torch.int32
            obs_t[k] = torch.as_tensor(v, dtype=dtype, device=self.device).unsqueeze(0)
        with torch.no_grad():
            scores, _ = self.model(obs_t)
        action = scores.argmax(dim=-1).item()
        return min(action, n_legal - 1)


class RandomLegal(Strategy):
    """Uniform random over legal actions."""

    def choose(self, obs: dict, n_legal: int) -> int:
        return int(np.random.randint(0, n_legal))


class FixedAction(Strategy):
    """Always choose a fixed action index (clamped to legal range)."""

    def __init__(self, index: int):
        self.index = index

    def choose(self, obs: dict, n_legal: int) -> int:
        return min(self.index, n_legal - 1)


class AlwaysFirst(Strategy):
    """Always pick action 0 (e.g., first offered card, or first legal play)."""

    def choose(self, obs: dict, n_legal: int) -> int:
        return 0


class AlwaysLast(Strategy):
    """Always pick the last legal action (typically 'skip' or 'end turn')."""

    def choose(self, obs: dict, n_legal: int) -> int:
        return n_legal - 1


# ---------------------------------------------------------------------------
# Composite router
# ---------------------------------------------------------------------------

class CompositeStrategy(Strategy):
    """Routes decisions to per-phase strategies.

    Args:
        default: Strategy used for any phase without an explicit override.
        **overrides: Phase-specific strategies keyed by phase name
                     (e.g., reward=AlwaysLast(), combat=ModelStrategy(...)).
    """

    def __init__(self, default: Strategy, **overrides: Strategy):
        self.default = default
        self._by_name = overrides
        self._by_id: dict[int, Strategy] = {}
        for name, strat in overrides.items():
            try:
                idx = PHASE_NAMES.index(name)
            except ValueError:
                raise ValueError(
                    f"Unknown phase '{name}'. Valid: {PHASE_NAMES}"
                )
            self._by_id[idx] = strat

    @property
    def trainable(self) -> bool:
        return self.default.trainable

    def choose(self, obs: dict, n_legal: int) -> int:
        phase_id = phase_from_obs(obs)
        strategy = self._by_id.get(phase_id, self.default)
        return strategy.choose(obs, n_legal)

    def strategy_for(self, obs: dict) -> Strategy:
        """Return the strategy that would handle this obs (for trainable checks)."""
        phase_id = phase_from_obs(obs)
        return self._by_id.get(phase_id, self.default)

    def is_trainable(self, obs: dict) -> bool:
        """Whether the active strategy for this obs produces training data."""
        return self.strategy_for(obs).trainable
