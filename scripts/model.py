"""Per-action scoring actor-critic for the Decker gauntlet.

The key idea: instead of scoring action *indices*, we score action *features*.
Each legal action gets a feature vector describing what it does (card played,
target enemy, etc.), and the network learns a shared scoring function.

This lets the network generalize:
  "attacking a low-HP enemy is good" works regardless of hand position.
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.distributions import Categorical

# Fallback dimensions for the unfiltered (full vocab) case.
# Actual values come from decker.GauntletEnv.instance_dims() at runtime
# and are passed to the DeckModel constructor. With a synergy filter active,
# these will be smaller (e.g. VOCAB_SIZE ~22 for marked-only dueling).
GLOBAL_DIM = 1615
CARD_FEAT_DIM = 314
ENEMY_FEAT_DIM = 54
ACTION_FEAT_DIM = 652
VOCAB_SIZE = 262

CARD_EMBED_DIM = 16
HIDDEN_DIM = 128
PHASE_OFFSET = 8
PHASE_COUNT = 7
COMBAT_PHASE_INDEX = 0


class DeckModel(nn.Module):
    """Actor-critic with per-action scoring.

    Architecture:
        - Global encoder:  Linear(global_dim, hidden) -> ReLU
        - Card encoder:    Embedding(vocab, embed_dim) + Linear(card_feats, hidden) -> ReLU
        - Enemy encoder:   Linear(enemy_feats, hidden) -> ReLU
        - Action scorer:   Linear(action_feats + hidden, hidden) -> ReLU -> Linear(1)
        - Value head:      pool(cards) + pool(enemies) + global -> Linear(1)
    """

    def __init__(
        self,
        global_dim=GLOBAL_DIM,
        card_feat_dim=CARD_FEAT_DIM,
        enemy_feat_dim=ENEMY_FEAT_DIM,
        action_feat_dim=ACTION_FEAT_DIM,
        vocab_size=VOCAB_SIZE,
        card_embed_dim=CARD_EMBED_DIM,
        hidden_dim=HIDDEN_DIM,
    ):
        super().__init__()
        self.global_dim = global_dim
        self.card_feat_dim = card_feat_dim
        self.enemy_feat_dim = enemy_feat_dim
        self.action_feat_dim = action_feat_dim
        self.vocab_size = vocab_size
        self.hidden_dim = hidden_dim

        # Global state encoder.
        self.global_enc = nn.Sequential(
            nn.Linear(global_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
        )

        # Card encoder (shared across all cards in hand).
        # Card features = continuous features (card_feat_dim - vocab_size) + embedding.
        self.card_embed = nn.Embedding(vocab_size, card_embed_dim)
        card_cont_dim = card_feat_dim - vocab_size  # 30
        self.card_enc = nn.Sequential(
            nn.Linear(card_cont_dim + card_embed_dim, hidden_dim),
            nn.ReLU(),
        )

        # Enemy encoder (shared across all enemies).
        self.enemy_enc = nn.Sequential(
            nn.Linear(enemy_feat_dim, hidden_dim),
            nn.ReLU(),
        )

        # Use separate final heads for combat and non-combat decisions.
        self.combat_action_scorer = nn.Sequential(
            nn.Linear(action_feat_dim + hidden_dim * 3, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )
        self.noncombat_action_scorer = nn.Sequential(
            nn.Linear(action_feat_dim + hidden_dim * 3, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )

        self.combat_value_head = nn.Sequential(
            nn.Linear(hidden_dim * 3, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )
        self.noncombat_value_head = nn.Sequential(
            nn.Linear(hidden_dim * 3, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )

    def _encode_cards(self, hand, n_hand):
        """Encode hand cards. hand: (B, MAX_HAND, CARD_FEAT_DIM)."""
        B = hand.shape[0]

        # Split continuous features from one-hot identity.
        # One-hot is the last VOCAB_SIZE features.
        cont = hand[:, :, :-self.vocab_size]  # (B, MAX_HAND, card_feat_dim - vocab_size)
        onehot = hand[:, :, -self.vocab_size:]  # (B, MAX_HAND, vocab_size)

        # Convert one-hot to index for embedding lookup.
        vocab_idx = onehot.argmax(dim=-1).clamp(0, self.card_embed.num_embeddings - 1)
        embed = self.card_embed(vocab_idx)  # (B, MAX_HAND, card_embed_dim)
        combined = torch.cat([cont, embed], dim=-1)

        card_embeds = self.card_enc(combined)  # (B, MAX_HAND, hidden)

        # Masked mean pool.
        mask = torch.arange(hand.shape[1], device=hand.device).unsqueeze(0) < n_hand.unsqueeze(1)
        mask = mask.unsqueeze(-1).float()  # (B, MAX_HAND, 1)
        pooled = (card_embeds * mask).sum(dim=1) / mask.sum(dim=1).clamp(min=1)
        return pooled  # (B, hidden)

    def _encode_enemies(self, enemies, n_enemies):
        """Encode enemies. enemies: (B, MAX_ENEMIES, ENEMY_FEAT_DIM)."""
        enemy_embeds = self.enemy_enc(enemies)  # (B, MAX_ENEMIES, hidden)

        mask = torch.arange(enemies.shape[1], device=enemies.device).unsqueeze(0) < n_enemies.unsqueeze(1)
        mask = mask.unsqueeze(-1).float()
        pooled = (enemy_embeds * mask).sum(dim=1) / mask.sum(dim=1).clamp(min=1)
        return pooled  # (B, hidden)

    def _phase_ids(self, global_f):
        phase_slice = global_f[:, PHASE_OFFSET : PHASE_OFFSET + PHASE_COUNT]
        return phase_slice.argmax(dim=-1)

    def _score_actions(self, action_feats, state_context, action_mask, phase_ids):
        """Score each action. action_feats: (B, MAX_ACTIONS, ACTION_FEAT_DIM)."""
        B, A, _ = action_feats.shape

        # Expand pooled state context to each action slot.
        context_exp = state_context.unsqueeze(1).expand(B, A, -1)
        combined = torch.cat([action_feats, context_exp], dim=-1)

        combat_scores = self.combat_action_scorer(combined).squeeze(-1)
        noncombat_scores = self.noncombat_action_scorer(combined).squeeze(-1)
        combat_mask = (phase_ids == COMBAT_PHASE_INDEX).unsqueeze(-1)
        scores = torch.where(combat_mask, combat_scores, noncombat_scores)

        # Mask invalid actions with large negative value.
        scores = scores + (1 - action_mask) * (-1e8)
        return scores

    def forward(self, obs):
        """Returns (action_log_probs, value, entropy) for the batch."""
        global_f = obs["global"]
        hand = obs["hand"]
        enemies = obs["enemies"]
        action_feats = obs["action_feats"]
        action_mask = obs["action_mask"]
        n_hand = obs["n_hand"]
        n_enemies = obs["n_enemies"]

        # Encode state.
        global_embed = self.global_enc(global_f)
        card_pool = self._encode_cards(hand, n_hand)
        enemy_pool = self._encode_enemies(enemies, n_enemies)
        phase_ids = self._phase_ids(global_f)

        value_input = torch.cat([global_embed, card_pool, enemy_pool], dim=-1)
        combat_value = self.combat_value_head(value_input).squeeze(-1)
        noncombat_value = self.noncombat_value_head(value_input).squeeze(-1)
        value = torch.where(phase_ids == COMBAT_PHASE_INDEX, combat_value, noncombat_value)

        # Share the same full state context with the actor.
        state_context = torch.cat([global_embed, card_pool, enemy_pool], dim=-1)

        # Action scores.
        scores = self._score_actions(action_feats, state_context, action_mask, phase_ids)

        return scores, value

    def get_action_and_value(self, obs, action=None):
        """Sample or evaluate an action.

        Returns: (action, log_prob, entropy, value)
        """
        scores, value = self.forward(obs)
        probs = F.softmax(scores, dim=-1)
        dist = Categorical(probs)

        if action is None:
            action = dist.sample()

        log_prob = dist.log_prob(action)
        entropy = dist.entropy()

        return action, log_prob, entropy, value

    def get_value(self, obs):
        """Just the value estimate."""
        _, value = self.forward(obs)
        return value
