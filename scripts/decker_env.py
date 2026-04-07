"""Gymnasium environment wrapping the Decker gauntlet via PyO3 bindings.

The environment returns structured observations as numpy arrays, suitable
for the per-action scoring architecture defined in model.py.
"""

import gymnasium as gym
import numpy as np
from gymnasium import spaces

import decker_pyenv as decker

# Default feature dimensions (full vocab). Override with instance_dims()
# when using a synergy filter or subset content.
_DEFAULT_DIMS = decker.GauntletEnv.feature_dims()
GLOBAL_DIM, CARD_FEAT_DIM, ENEMY_FEAT_DIM, ACTION_FEAT_DIM, VOCAB_SIZE = _DEFAULT_DIMS

def instance_dims(subclass="two_handed", race="human", synergy_filter=None):
    """Get actual feature dims for a specific config (may differ from defaults)."""
    vec = decker.VecGauntletEnv(subclass, [0], race, synergy_filter=synergy_filter)
    dims = vec.instance_dims()
    del vec
    return dims

MAX_HAND = 12
MAX_ENEMIES = 4
MAX_ACTIONS = 64


class DeckEnv(gym.Env):
    """Gauntlet environment with structured observations for RL.

    Observation space is a Dict:
        - global:       (GLOBAL_DIM,)        normalized player/game state
        - hand:         (MAX_HAND, CARD_FEAT_DIM)   per-card features (zero-padded)
        - enemies:      (MAX_ENEMIES, ENEMY_FEAT_DIM)  per-enemy features (zero-padded)
        - action_feats: (MAX_ACTIONS, ACTION_FEAT_DIM)  per-action features (zero-padded)
        - action_mask:  (MAX_ACTIONS,)       1.0 for valid actions, 0.0 for invalid
        - n_hand:       ()                   number of cards in hand
        - n_enemies:    ()                   number of alive enemies
        - n_actions:    ()                   number of legal actions

    Action space: Discrete(MAX_ACTIONS)
    """

    metadata = {"render_modes": []}

    def __init__(self, subclass="two_handed", seed=0, race="human", dims=None):
        super().__init__()
        self.subclass = subclass
        self._seed = seed
        self.race = race
        self._env = decker.GauntletEnv(subclass, seed, race)

        # Use provided dims or fall back to defaults.
        if dims is not None:
            g_dim, c_dim, e_dim, a_dim = dims[0], dims[1], dims[2], dims[3]
        else:
            g_dim, c_dim, e_dim, a_dim = GLOBAL_DIM, CARD_FEAT_DIM, ENEMY_FEAT_DIM, ACTION_FEAT_DIM
        self._g_dim = g_dim
        self._c_dim = c_dim
        self._e_dim = e_dim
        self._a_dim = a_dim

        self.observation_space = spaces.Dict(
            {
                "global": spaces.Box(-1.0, 10.0, shape=(g_dim,), dtype=np.float32),
                "hand": spaces.Box(
                    -1.0, 100.0, shape=(MAX_HAND, c_dim), dtype=np.float32
                ),
                "enemies": spaces.Box(
                    -1.0, 100.0, shape=(MAX_ENEMIES, e_dim), dtype=np.float32
                ),
                "action_feats": spaces.Box(
                    -1.0,
                    100.0,
                    shape=(MAX_ACTIONS, a_dim),
                    dtype=np.float32,
                ),
                "action_mask": spaces.Box(
                    0.0, 1.0, shape=(MAX_ACTIONS,), dtype=np.float32
                ),
                "n_hand": spaces.Box(0, MAX_HAND, shape=(), dtype=np.int32),
                "n_enemies": spaces.Box(0, MAX_ENEMIES, shape=(), dtype=np.int32),
                "n_actions": spaces.Box(0, MAX_ACTIONS, shape=(), dtype=np.int32),
            }
        )
        self.action_space = spaces.Discrete(MAX_ACTIONS)

    def _build_obs(self):
        gf = np.array(self._env.get_global_features(), dtype=np.float32)
        hf_raw = self._env.get_hand_features()
        ef_raw = self._env.get_enemy_features()
        af_raw = self._env.get_action_features()

        n_hand = len(hf_raw)
        n_enemies = len(ef_raw)
        n_actions = len(af_raw)

        hand = np.zeros((MAX_HAND, self._c_dim), dtype=np.float32)
        for i, c in enumerate(hf_raw[:MAX_HAND]):
            hand[i] = c

        enemies = np.zeros((MAX_ENEMIES, self._e_dim), dtype=np.float32)
        for i, e in enumerate(ef_raw[:MAX_ENEMIES]):
            enemies[i] = e

        action_feats = np.zeros((MAX_ACTIONS, self._a_dim), dtype=np.float32)
        action_mask = np.zeros(MAX_ACTIONS, dtype=np.float32)
        for i, a in enumerate(af_raw[:MAX_ACTIONS]):
            action_feats[i] = a
            action_mask[i] = 1.0

        return {
            "global": gf,
            "hand": hand,
            "enemies": enemies,
            "action_feats": action_feats,
            "action_mask": action_mask,
            "n_hand": np.int32(n_hand),
            "n_enemies": np.int32(n_enemies),
            "n_actions": np.int32(n_actions),
        }

    def reset(self, *, seed=None, options=None):
        if seed is not None:
            self._seed = seed
        else:
            self._seed += 1
        self._env.reset(self.subclass, self._seed, self.race)
        return self._build_obs(), {}

    def step(self, action):
        n_legal = int(self._build_obs()["n_actions"])
        if action >= n_legal:
            action = 0

        gf, hf, ef, af, reward, done, n_legal_new = self._env.step_features(action)

        obs = self._build_obs()
        return obs, reward, done, False, {"fights_won": self._env.fights_won}

    @property
    def fights_won(self):
        return self._env.fights_won
