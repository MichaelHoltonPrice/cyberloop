//! Observation — a snapshot of the gauntlet state for bots and RL agents.
//!
//! The [`Observation`] is the primary interface between the game state and
//! decision-making agents. It provides all the information needed to choose
//! an action without direct access to internal state.

use decker_engine::card::{CardDef, Effect};
use decker_engine::card_ids::CardId;
use decker_engine::class::CardTag;
use decker_engine::enemy::IntentType;
use decker_engine::status::StatusType;
use serde::{Deserialize, Serialize};

use crate::{GauntletPhase, GauntletRunner};

/// Pre-compute total base damage from a card's effects.
fn card_total_damage(def: &CardDef) -> i32 {
    def.effects
        .iter()
        .map(|e| match e {
            Effect::DealDamage { amount, .. } => *amount,
            Effect::DealDamageIfDamagedLastTurn { amount, .. } => *amount,
            Effect::DealDamageIfTargetDebuffed { amount, .. } => *amount,
            Effect::DealDamageIfEnemyBlocked { amount, .. } => *amount,
            Effect::DealDamageIfMarked { amount, .. } => *amount,
            Effect::DealDamagePerCardPlayed { per_card, .. } => *per_card,
            Effect::DealDamagePerMarkedStack { per_stack, .. } => *per_stack,
            Effect::DealDamageScaledByMarked {
                base_damage,
                per_stack,
                ..
            } => base_damage + per_stack, // approximate: 1 stack
            Effect::DealDamagePerDebuffOnTarget { base, per_debuff, .. } => base + per_debuff, // approximate: 1 debuff
            Effect::DealDamageScaledByMissingHp { base, .. } => *base,
            Effect::DealDamageScaledByMissingHpEnemy { base, .. } => *base,
            Effect::DealDamagePerPlayerStatus { per_stack, .. } => *per_stack,
            Effect::DealDamageSpendKi { base, .. } => *base,
            Effect::DealDamageSpendSmite { base, .. } => *base,
            Effect::DealDamageSpendArtifice { base, .. } => *base,
            Effect::DealDamageIfInWildShape { base, .. } => *base,
            Effect::DealDamageWithFormBonus { base, .. } => *base,
            Effect::DealDamageIfPlayerHasStatus { amount, .. } => *amount,
            _ => 0,
        })
        .sum()
}

/// Pre-compute total base block from a card's effects.
fn card_total_block(def: &CardDef) -> i32 {
    def.effects
        .iter()
        .map(|e| match e {
            Effect::GainBlock { amount } => *amount,
            Effect::GainBlockIfDamagedLastTurn { amount } => *amount,
            Effect::BlockEqualMissingHp { max } => *max,
            Effect::GainBlockPerStatusStack { multiplier, .. } => *multiplier,
            Effect::GainBlockConditional { amount, .. } => *amount,
            Effect::GainBlockSpendKi { base, .. } => *base,
            Effect::GainBlockPerKiStack { multiplier } => *multiplier, // approximate: 1 Ki
            Effect::GainBlockSpendSmite { base, .. } => *base,
            Effect::GainBlockWithFormBonus { base, .. } => *base,
            _ => 0,
        })
        .sum()
}

/// Pre-compute total draw count from a card's effects (net of discards).
fn card_draw_count(def: &CardDef) -> i32 {
    def.effects
        .iter()
        .map(|e| match e {
            Effect::DrawCards { count } => *count as i32,
            Effect::DrawAndDiscard { draw, discard } => *draw as i32 - *discard as i32,
            Effect::DrawPerEnemyWithStatus { .. } => 2, // avg ~2 enemies alive
            Effect::DrawCardsWithFormBonus { base_draw, .. } => *base_draw as i32,
            _ => 0,
        })
        .sum()
}

/// Pre-compute total energy gained from a card's effects.
fn card_energy_gain(def: &CardDef) -> i32 {
    def.effects
        .iter()
        .map(|e| match e {
            Effect::GainEnergy { amount } => *amount,
            _ => 0,
        })
        .sum()
}

/// Collect per-status stacks applied to enemies.
fn card_enemy_statuses(def: &CardDef) -> Vec<(StatusType, i32)> {
    use decker_engine::card::Target;
    let mut map: std::collections::HashMap<StatusType, i32> = std::collections::HashMap::new();
    for e in &def.effects {
        match e {
            Effect::ApplyStatus {
                target: Target::SingleEnemy | Target::AllEnemies | Target::RandomEnemy,
                status,
                stacks,
            }
            => {
                *map.entry(*status).or_insert(0) += stacks;
            }
            Effect::ApplyStatusPerCardPlayed {
                target: Target::SingleEnemy | Target::AllEnemies | Target::RandomEnemy,
                status,
                per_card,
            } => {
                *map.entry(*status).or_insert(0) += per_card;
            }
            _ => {}
        }
    }
    map.into_iter().collect()
}

/// Collect per-status stacks applied to self (player).
fn card_self_statuses(def: &CardDef) -> Vec<(StatusType, i32)> {
    use decker_engine::card::Target;
    let mut map: std::collections::HashMap<StatusType, i32> = std::collections::HashMap::new();
    for e in &def.effects {
        match e {
            Effect::ApplyStatus {
                target: Target::Player,
                status,
                stacks,
            }
            => {
                *map.entry(*status).or_insert(0) += stacks;
            }
            Effect::ApplyStatusPerCardPlayed {
                target: Target::Player,
                status,
                per_card,
            } => {
                *map.entry(*status).or_insert(0) += per_card;
            }
            _ => {}
        }
    }
    map.into_iter().collect()
}

/// Whether the card has any conditional effects (damage/block depends on game state).
fn card_has_conditional(def: &CardDef) -> bool {
    def.effects.iter().any(|e| {
        matches!(
            e,
            Effect::DealDamageIfDamagedLastTurn { .. }
                | Effect::DealDamageIfTargetDebuffed { .. }
                | Effect::DealDamageIfEnemyBlocked { .. }
                | Effect::DealDamageIfMarked { .. }
                | Effect::DealDamagePerCardPlayed { .. }
                | Effect::DealDamageEqualBlock { .. }
                | Effect::GainBlockIfDamagedLastTurn { .. }
                | Effect::BlockEqualMissingHp { .. }
                | Effect::GainBlockConditional { .. }
                | Effect::DealDamageScaledByMarked { .. }
                | Effect::DealDamagePerDebuffOnTarget { .. }
                | Effect::DealDamageScaledByMissingHp { .. }
                | Effect::DealDamageScaledByMissingHpEnemy { .. }
                | Effect::DealDamagePerPlayerStatus { .. }
                | Effect::DealDamageSpendKi { .. }
                | Effect::GainBlockSpendKi { .. }
                | Effect::GainBlockPerKiStack { .. }
                | Effect::DealDamageSpendSmite { .. }
                | Effect::GainBlockSpendSmite { .. }
                | Effect::DealDamageSpendArtifice { .. }
                | Effect::DealDamageIfInWildShape { .. }
                | Effect::DealDamageWithFormBonus { .. }
                | Effect::GainBlockWithFormBonus { .. }
                | Effect::DrawCardsWithFormBonus { .. }
                | Effect::DealDamageIfPlayerHasStatus { .. }
                | Effect::ApplyStatusIfPlayerHasStatus { .. }
        )
    })
}

/// Whether any effect targets all enemies (AoE).
fn card_targets_all(def: &CardDef) -> bool {
    use decker_engine::card::Target;
    def.effects.iter().any(|e| {
        matches!(
            e,
            Effect::DealDamage {
                target: Target::AllEnemies,
                ..
            } | Effect::ApplyStatus {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageIfDamagedLastTurn {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageIfTargetDebuffed {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageIfEnemyBlocked {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageEqualBlock {
                target: Target::AllEnemies
            } | Effect::DealDamagePerCardPlayed {
                target: Target::AllEnemies,
                ..
            } | Effect::RemoveEnemyBlock {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamagePerMarkedStack {
                target: Target::AllEnemies,
                ..
            } | Effect::ApplyStatusPerCardPlayed {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageScaledByMarked {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamagePerDebuffOnTarget {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageScaledByMissingHp {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageScaledByMissingHpEnemy {
                target: Target::AllEnemies,
                ..
            }
        )
    })
}

/// Net HP change from a card (heal minus self-damage).
fn card_hp_change(def: &CardDef) -> i32 {
    def.effects
        .iter()
        .map(|e| match e {
            Effect::Heal { amount } => *amount,
            Effect::LoseHp { amount } => -*amount,
            _ => 0,
        })
        .sum()
}

// ── Sub-observations ─────────────────────────────────────────────────────

/// Observable information about a card in hand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardObs {
    pub card_id: CardId,
    pub name: String,
    pub cost: i32,
    pub tags: Vec<CardTag>,
    pub description: String,
    /// Whether the card can currently be played (enough energy, valid targets, etc.).
    pub playable: bool,
    /// Pre-computed total base damage across all effects.
    pub total_damage: i32,
    /// Pre-computed total base block across all effects.
    pub total_block: i32,
    /// Whether the card is removed from the deck after play.
    pub exhaust: bool,
    /// Total cards drawn by this card's effects.
    pub draw_count: i32,
    /// Total energy gained from this card's effects.
    pub energy_gain: i32,
    /// Per-status stacks applied to enemies.
    pub enemy_statuses: Vec<(StatusType, i32)>,
    /// Per-status stacks applied to self (player).
    pub self_statuses: Vec<(StatusType, i32)>,
    /// Whether this card targets all enemies (AoE).
    pub targets_all: bool,
    /// Net HP change (heal minus self-damage).
    pub hp_change: i32,
    /// Whether the card is drawn on turn 1.
    pub innate: bool,
    /// Whether the card goes to draw pile bottom instead of discard.
    pub recycle: bool,
    /// Whether the card has conditional effects (e.g. "if damaged last turn").
    pub has_conditional: bool,
    /// Whether the card stays in hand at end of turn (concentration).
    #[serde(default)]
    pub concentration: bool,
}

/// Observable information about an enemy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyObs {
    pub index: usize,
    pub enemy_id: String,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub intent_index: usize,
    pub intent: Option<IntentType>,
    pub statuses: Vec<(StatusType, i32)>,
    pub alive: bool,
}

// ── Observed phase ──────────────────────────────────────────────────────

/// High-level phase for RL agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservedPhase {
    Combat,
    Reward,
    CollectionOverflow,
    InnateChoice,
    DeckSwap,
    DeckRebuild,
    GameOver,
}

// ── Main observation ─────────────────────────────────────────────────────

/// A complete snapshot of the gauntlet state for decision-making.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    // ── Phase (backward compat) ─────────────────────────────────────
    pub in_combat: bool,
    pub in_reward: bool,
    pub game_over: bool,

    // ── Phase (new) ─────────────────────────────────────────────────
    pub phase_type: ObservedPhase,

    // ── Player ───────────────────────────────────────────────────────
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub player_block: i32,
    pub player_energy: i32,
    pub player_max_energy: i32,
    pub player_statuses: Vec<(StatusType, i32)>,
    pub player_level: u32,

    // ── Hand (combat only) ───────────────────────────────────────────
    pub hand: Vec<CardObs>,

    // ── Enemies (combat only) ────────────────────────────────────────
    pub enemies: Vec<EnemyObs>,

    // ── Pile sizes (combat only) ─────────────────────────────────────
    pub draw_pile_size: usize,
    pub discard_pile_size: usize,
    pub turn: u32,
    pub block_floor: i32,
    pub retain_block_cap: i32,
    /// Number of cards the player must discard before taking other actions.
    pub pending_discards: usize,

    // ── Pile card IDs (combat only) ──────────────────────────────────
    pub draw_pile_card_ids: Vec<CardId>,
    pub hand_card_ids: Vec<CardId>,
    pub discard_pile_card_ids: Vec<CardId>,
    pub exhaust_pile_card_ids: Vec<CardId>,

    // ── Run progress ─────────────────────────────────────────────────
    pub fights_won: u32,
    pub play_deck_size: usize,
    pub play_deck_card_ids: Vec<CardId>,
    pub collection_size: usize,
    pub collection_card_ids: Vec<CardId>,

    // ── Perk / progression ───────────────────────────────────────────
    pub bonus_draw: i32,
    pub bonus_energy: i32,
    pub innate_card_ids: Vec<CardId>,

    // ── Reward/offer options ────────────────────────────────────────
    pub reward_cards: Vec<CardObs>,

    // ── Acquired card (DeckSwap phase) ───────────────────────────────
    /// The newly acquired card during DeckSwap (from `new_card_id`).
    pub acquired_card: Option<CardObs>,

    // ── Current choice set (overflow trim / ordered rebuild) ──────────
    /// Cards directly selectable in the current phase.
    pub choice_cards: Vec<CardObs>,

    // ── Ordered rebuild state ─────────────────────────────────────────
    pub rebuild_slot_index: usize,
    pub rebuild_partial_deck_card_ids: Vec<CardId>,
    pub rebuild_remaining_card_ids: Vec<CardId>,

    // ── Combat conditionals ───────────────────────────────────────────
    /// True if the player took damage during the previous turn this combat.
    /// Used by bots to determine if "if damaged last turn" conditionals will fire.
    pub player_took_damage_last_turn: bool,

    // ── Cards played (combat only) ──────────────────────────────────
    /// Number of cards played during the current player turn.
    #[serde(default)]
    pub cards_played_this_turn: u32,

    // ── Play deck detail ────────────────────────────────────────────
    /// Full CardObs for each card in the play deck (for DeckSwap/InnateChoice evaluation).
    #[serde(default)]
    pub play_deck_cards: Vec<CardObs>,

    // ── Character identity ──────────────────────────────────────────
    /// The player's subclass ID (e.g. "two_handed", "defense", "dueling").
    #[serde(default)]
    pub subclass: String,
    /// The player's race (e.g. "human", "elf", "dwarf").
    #[serde(default)]
    pub race: String,
    /// The player's background (e.g. "soldier", "acolyte").
    #[serde(default)]
    pub background: String,
}

/// Build a [`CardObs`] from a card definition.
fn card_obs_from_def(card_id: &CardId, def: Option<&CardDef>, playable: bool) -> CardObs {
    CardObs {
        card_id: card_id.clone(),
        name: def.map(|d| d.name.clone()).unwrap_or_default(),
        cost: def.map(|d| d.cost).unwrap_or(0),
        tags: def.map(|d| d.tags.clone()).unwrap_or_default(),
        description: def.map(|d| d.description.clone()).unwrap_or_default(),
        playable,
        total_damage: def.map(card_total_damage).unwrap_or(0),
        total_block: def.map(card_total_block).unwrap_or(0),
        exhaust: def.map(|d| d.exhaust).unwrap_or(false),
        draw_count: def.map(card_draw_count).unwrap_or(0),
        energy_gain: def.map(card_energy_gain).unwrap_or(0),
        enemy_statuses: def.map(card_enemy_statuses).unwrap_or_default(),
        self_statuses: def.map(card_self_statuses).unwrap_or_default(),
        targets_all: def.map(card_targets_all).unwrap_or(false),
        hp_change: def.map(card_hp_change).unwrap_or(0),
        innate: def.map(|d| d.innate).unwrap_or(false),
        recycle: def.map(|d| d.recycle).unwrap_or(false),
        has_conditional: def.map(card_has_conditional).unwrap_or(false),
        concentration: def.map(|d| d.concentration).unwrap_or(false),
    }
}

impl GauntletRunner {
    /// Build a snapshot of the current game state for decision-making.
    pub fn observe(&self) -> Observation {
        let (in_combat, in_reward, game_over) = match &self.phase {
            GauntletPhase::Combat => (true, false, false),
            GauntletPhase::Reward => (false, true, false),
            GauntletPhase::GameOver => (false, false, true),
            _ => (false, false, false),
        };

        let phase_type = match &self.phase {
            GauntletPhase::Combat => ObservedPhase::Combat,
            GauntletPhase::Reward => ObservedPhase::Reward,
            GauntletPhase::CollectionOverflow { .. } => ObservedPhase::CollectionOverflow,
            GauntletPhase::InnateChoice => ObservedPhase::InnateChoice,
            GauntletPhase::DeckSwap { .. } => ObservedPhase::DeckSwap,
            GauntletPhase::DeckRebuild { .. } => ObservedPhase::DeckRebuild,
            GauntletPhase::GameOver => ObservedPhase::GameOver,
        };

        let (
            hand,
            enemies,
            draw_pile_size,
            discard_pile_size,
            turn,
            player_hp,
            player_max_hp,
            player_block,
            player_energy,
            player_max_energy,
            player_statuses,
            draw_pile_card_ids,
            hand_card_ids,
            discard_pile_card_ids,
            exhaust_pile_card_ids,
            block_floor,
            retain_block_cap,
            pending_discards,
            player_took_damage_last_turn,
            cards_played_this_turn,
        ) = if let Some(combat) = &self.combat {
            let hand: Vec<CardObs> = combat
                .hand
                .iter()
                .enumerate()
                .map(|(i, card)| {
                    let def = self.content.card_defs.get(&card.def_id);
                    card_obs_from_def(&card.def_id, def, combat.can_play_card(i, &self.content))
                })
                .collect();

            let enemies: Vec<EnemyObs> = combat
                .enemies
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let name = self
                        .content
                        .enemy_defs
                        .get(&e.def_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_default();
                    let statuses: Vec<(StatusType, i32)> = e
                        .status_effects
                        .iter()
                        .map(|(&st, &val)| (st, val))
                        .collect();
                    EnemyObs {
                        index: i,
                        enemy_id: e.def_id.clone(),
                        name,
                        hp: e.hp,
                        max_hp: e.max_hp,
                        block: e.block,
                        intent_index: e.intent_index,
                        intent: e.current_intent.clone(),
                        statuses,
                        alive: !e.is_dead(),
                    }
                })
                .collect();

            let player_statuses: Vec<(StatusType, i32)> = combat
                .player
                .status_effects
                .iter()
                .map(|(&st, &val)| (st, val))
                .collect();

            let draw_ids: Vec<CardId> = combat.draw_pile.iter().map(|c| c.def_id.clone()).collect();
            let hand_ids: Vec<CardId> = combat.hand.iter().map(|c| c.def_id.clone()).collect();
            let discard_ids: Vec<CardId> =
                combat.discard.iter().map(|c| c.def_id.clone()).collect();
            let exhaust_ids: Vec<CardId> =
                combat.exhaust.iter().map(|c| c.def_id.clone()).collect();

            (
                hand,
                enemies,
                combat.draw_pile.len(),
                combat.discard.len(),
                combat.turn,
                combat.player.hp,
                combat.player.max_hp,
                combat.player.block,
                combat.player.energy,
                combat.player.max_energy,
                player_statuses,
                draw_ids,
                hand_ids,
                discard_ids,
                exhaust_ids,
                combat.block_floor,
                combat.retain_block_cap,
                combat.pending_discards,
                combat.player_took_damage_last_turn,
                combat.cards_played_this_turn,
            )
        } else {
            let player_statuses: Vec<(StatusType, i32)> = self
                .player
                .status_effects
                .iter()
                .map(|(&st, &val)| (st, val))
                .collect();
            (
                Vec::new(),
                Vec::new(),
                0,
                0,
                0,
                self.player.hp,
                self.player.max_hp,
                self.player.block,
                self.player.energy,
                self.player.max_energy,
                player_statuses,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                0,
                0,
                0,
                false,
                0,
            )
        };

        let reward_cards = self
            .reward_options
            .iter()
            .map(|card_id| {
                let def = self.content.card_defs.get(card_id);
                card_obs_from_def(card_id, def, true)
            })
            .collect();

        let play_deck_card_ids: Vec<CardId> = self.play_deck.iter().map(|c| c.def_id.clone()).collect();
        let play_deck_cards: Vec<CardObs> = self
            .play_deck
            .iter()
            .map(|card| {
                let def = self.content.card_defs.get(&card.def_id);
                card_obs_from_def(&card.def_id, def, false)
            })
            .collect();
        let collection_card_ids = self.collection.iter().map(|c| c.def_id.clone()).collect();
        let innate_card_ids: Vec<CardId> = self.innate_overrides.iter().cloned().collect();

        // Acquired card (DeckSwap phase only).
        let acquired_card = if let GauntletPhase::DeckSwap { new_card_id } = &self.phase {
            let def = self.content.card_defs.get(new_card_id);
            Some(card_obs_from_def(new_card_id, def, false))
        } else {
            None
        };

        let choice_cards = match &self.phase {
            GauntletPhase::CollectionOverflow { .. } => self
                .overflow_removal_indices()
                .iter()
                .filter_map(|&idx| self.collection.get(idx))
                .map(|card| {
                    let def = self.content.card_defs.get(&card.def_id);
                    card_obs_from_def(&card.def_id, def, false)
                })
                .collect(),
            GauntletPhase::DeckRebuild { partial_deck, .. } => self
                .rebuild_card_choices(partial_deck)
                .iter()
                .map(|card_id| {
                    let def = self.content.card_defs.get(card_id);
                    card_obs_from_def(card_id, def, false)
                })
                .collect(),
            _ => Vec::new(),
        };

        let (rebuild_slot_index, rebuild_partial_deck_card_ids, rebuild_remaining_card_ids) =
            match &self.phase {
                GauntletPhase::DeckRebuild {
                    slot_idx,
                    partial_deck,
                } => {
                    let partial_ids: Vec<CardId> = partial_deck
                        .iter()
                        .map(|card| card.def_id.clone())
                        .collect();
                    let mut remaining_counts =
                        crate::GauntletRunner::collection_counts(&self.collection);
                    for card in partial_deck {
                        if let Some(count) = remaining_counts.get_mut(&card.def_id) {
                            *count = count.saturating_sub(1);
                        }
                    }
                    let mut remaining_ids = Vec::new();
                    for card in &self.collection {
                        if let Some(count) = remaining_counts.get_mut(&card.def_id) {
                            if *count > 0 {
                                remaining_ids.push(card.def_id.clone());
                                *count -= 1;
                            }
                        }
                    }
                    (*slot_idx, partial_ids, remaining_ids)
                }
                _ => (0, Vec::new(), Vec::new()),
            };

        Observation {
            in_combat,
            in_reward,
            game_over,
            phase_type,
            player_hp,
            player_max_hp,
            player_block,
            player_energy,
            player_max_energy,
            player_statuses,
            player_level: self.level,
            hand,
            enemies,
            draw_pile_size,
            discard_pile_size,
            turn,
            block_floor,
            retain_block_cap,
            pending_discards,
            draw_pile_card_ids,
            hand_card_ids,
            discard_pile_card_ids,
            exhaust_pile_card_ids,
            fights_won: self.fights_won,
            play_deck_size: self.play_deck.len(),
            play_deck_card_ids,
            collection_size: self.collection.len(),
            collection_card_ids,
            bonus_draw: self.bonus_draw,
            bonus_energy: self.bonus_energy,
            innate_card_ids,
            reward_cards,
            acquired_card,
            choice_cards,
            rebuild_slot_index,
            rebuild_partial_deck_card_ids,
            rebuild_remaining_card_ids,
            player_took_damage_last_turn,
            cards_played_this_turn,
            play_deck_cards,
            subclass: self.subclass_id.clone(),
            race: self.race.clone(),
            background: self.background.clone(),
        }
    }
}
