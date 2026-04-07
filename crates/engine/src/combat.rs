//! Combat state machine — the core of the Decker engine.
//!
//! The engine is driven by **Actions** (player/agent decisions) and produces
//! **Events** (observable consequences). The fundamental operation:
//!
//! ```ignore
//! let events = combat.apply(action, &content)?;
//! ```
//!
//! State mutates in place. The returned events describe what happened.
//! Between two actions there are no decisions — everything is deterministic
//! given the RNG seed.

use serde::{Deserialize, Serialize};

use crate::card::{CardInstance, CardType, Effect, Target};
use crate::card_ids::CardId;
use crate::class::{CardTag, Class};
use crate::content_tables::ContentTables;
use crate::enemy::{EnemyDef, EnemyState, IntentType};
use crate::ids::EnemyId;
use crate::player::PlayerState;
use crate::rng::GameRng;
use crate::status::{StatusMap, StatusType};

/// Maximum number of cards the player can hold in hand.
///
/// Draws stop once the cap is reached; effects that would add cards to a full
/// hand route the card to the discard pile instead.
const MAX_HAND_SIZE: usize = 12;

/// If a fight exceeds this many turns, it counts as a defeat.
/// Prevents infinite stalemates from block-heavy decks that can survive
/// but never deal enough damage to kill enemies.
const MAX_TURNS: u32 = 200;

/// Calculate Arcane Charge bonus damage and whether to draw a card.
/// 1 charge = +1, 2 = +3, 3 = +5, 4+ = +5 + draw 1.
fn arcane_charge_bonus(charges: i32) -> (i32, bool) {
    match charges {
        0 => (0, false),
        1 => (1, false),
        2 => (3, false),
        3 => (5, false),
        _ => (5, true), // 4+ charges
    }
}

// ---------------------------------------------------------------------------
// Combat phase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatPhase {
    PlayerTurn,
    EnemyTurn,
    Victory,
    Defeat,
}

// ---------------------------------------------------------------------------
// Actions — player/agent decisions (inputs to the state machine)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Play a card from hand, optionally targeting a specific enemy.
    PlayCard {
        hand_index: usize,
        target: Option<usize>,
    },
    /// End the player's turn, triggering enemy actions and a new turn.
    EndTurn,
    /// Discard a card from hand (required by DrawAndDiscard effects).
    DiscardCard { hand_index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    InvalidHandIndex,
    InsufficientEnergy,
    InvalidTarget,
    NotPlayerTurn,
    CombatOver,
    /// A pending discard must be resolved before playing cards or ending turn.
    MustDiscardFirst,
    /// Tried to discard when no discards are pending.
    NoPendingDiscards,
}

// ---------------------------------------------------------------------------
// Events — observable consequences (outputs of a state transition)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventTarget {
    Player,
    Enemy(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    // ── Card lifecycle ──────────────────────────────────────────────────
    CardPlayed {
        card_id: CardId,
        hand_index: usize,
        energy_spent: i32,
    },
    CardsDrawn {
        count: usize,
    },
    CardDiscarded {
        card_id: CardId,
    },
    CardExhausted {
        card_id: CardId,
    },

    // ── Damage & healing ───────────────────────────────────────────────
    DamageDealt {
        target: EventTarget,
        base: i32,
        modified: i32,
        blocked: i32,
        hp_lost: i32,
    },
    DotDamage {
        target: EventTarget,
        source: StatusType,
        damage: i32,
    },
    Healed {
        target: EventTarget,
        amount: i32,
        new_hp: i32,
    },

    // ── Block ──────────────────────────────────────────────────────────
    BlockGained {
        target: EventTarget,
        amount: i32,
    },

    // ── Status effects ─────────────────────────────────────────────────
    StatusApplied {
        target: EventTarget,
        status: StatusType,
        stacks: i32,
        new_total: i32,
    },

    // ── Energy ─────────────────────────────────────────────────────────
    EnergyGained {
        amount: i32,
    },

    // ── Enemy actions ──────────────────────────────────────────────────
    EnemyAction {
        enemy_index: usize,
        intent: IntentType,
    },
    EnemySkipped {
        enemy_index: usize,
    },
    EnemyDied {
        enemy_index: usize,
    },

    // ── Combat lifecycle ───────────────────────────────────────────────
    TurnStarted {
        turn: u32,
    },
    TurnEnded {
        turn: u32,
    },
    PlayerDied,
    CombatVictory,
    CombatDefeat,
}

// ---------------------------------------------------------------------------
// Enemy scaling
// ---------------------------------------------------------------------------

/// How to interpret enemy definitions during combat.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnemyScaling {
    /// Narrative/act scaling; `boss` uses separate multipliers.
    Act { act: u32, boss: bool },
    /// Custom multipliers (gauntlet progression).
    Custom { hp_mult: f64, dmg_mult: f64 },
}

// ---------------------------------------------------------------------------
// CombatState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub draw_pile: Vec<CardInstance>,
    pub hand: Vec<CardInstance>,
    pub discard: Vec<CardInstance>,
    pub exhaust: Vec<CardInstance>,
    pub player: PlayerState,
    pub enemies: Vec<EnemyState>,
    pub phase: CombatPhase,
    pub turn: u32,
    pub rng: GameRng,
    pub class: Option<Class>,
    pub player_took_damage_last_turn: bool,
    /// True once a concentration card has been played this combat.
    pub concentration_active: bool,
    /// Cards the player must discard before acting (set by DrawAndDiscard).
    pub pending_discards: usize,
    pub act: u32,
    pub enemy_scaling: EnemyScaling,
    /// Number of cards played during the current player turn.
    pub cards_played_this_turn: u32,
    /// Block cannot go below this value at start of turn (Unbreakable).
    pub block_floor: i32,
    /// Retain up to this much block permanently (Iron Fortress). 0 = disabled.
    pub retain_block_cap: i32,
    /// Number of cards drawn per turn (default 5).
    pub draw_per_turn: usize,
    /// Set of card IDs that should be treated as innate (from InnateChoice perk).
    #[serde(default)]
    pub innate_overrides: std::collections::HashSet<CardId>,
    /// True once an Attack card has been played this turn (for WildShapeEagle first-attack draw).
    #[serde(default)]
    pub eagle_attack_used: bool,
}

impl CombatState {
    // ── Constructors ────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        player: PlayerState,
        deck: Vec<CardInstance>,
        enemies: Vec<EnemyState>,
        seed: u64,
        bonus_draw_turn1: usize,
        bonus_energy_turn1: usize,
        class: Option<Class>,
        content: &ContentTables,
    ) -> Self {
        Self::new_with_act(
            player,
            deck,
            enemies,
            seed,
            bonus_draw_turn1,
            bonus_energy_turn1,
            class,
            1,
            EnemyScaling::Act {
                act: 1,
                boss: false,
            },
            content,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_act(
        player: PlayerState,
        deck: Vec<CardInstance>,
        enemies: Vec<EnemyState>,
        seed: u64,
        bonus_draw_turn1: usize,
        bonus_energy_turn1: usize,
        class: Option<Class>,
        act: u32,
        enemy_scaling: EnemyScaling,
        content: &ContentTables,
    ) -> Self {
        let mut rng = GameRng::new(seed);
        let mut draw_pile = deck;
        rng.shuffle(&mut draw_pile);

        let mut state = Self {
            draw_pile,
            hand: Vec::new(),
            discard: Vec::new(),
            exhaust: Vec::new(),
            player,
            enemies,
            phase: CombatPhase::PlayerTurn,
            turn: 1,
            rng,
            class,
            player_took_damage_last_turn: false,
            concentration_active: false,
            pending_discards: 0,
            act,
            enemy_scaling,
            cards_played_this_turn: 0,
            block_floor: 0,
            retain_block_cap: 0,
            draw_per_turn: 5,
            innate_overrides: std::collections::HashSet::new(),
            eagle_attack_used: false,
        };

        // Move innate cards to the top of draw pile so they're drawn first.
        let mut innate = Vec::new();
        let mut rest = Vec::new();
        for card in state.draw_pile.drain(..) {
            let is_innate = content
                .card_defs
                .get(&card.def_id)
                .map(|d| d.innate)
                .unwrap_or(false)
                || state.innate_overrides.contains(&card.def_id);
            if is_innate {
                innate.push(card);
            } else {
                rest.push(card);
            }
        }
        state.draw_pile = rest;
        state.draw_pile.append(&mut innate);

        // Paladin: start combat with 2 Smite
        if class == Some(Class::Paladin) {
            state.player.status_effects.apply(StatusType::Smite, 2);
        }

        let mut setup_events = Vec::new();
        state.draw_cards(state.draw_per_turn + bonus_draw_turn1, &mut setup_events);
        state.player.energy += bonus_energy_turn1 as i32;

        // Rogue starts combat with 10 Artifice.
        if state.class == Some(Class::Rogue) {
            state.player.status_effects.apply(StatusType::Artifice, 10);
        }

        state
    }

    // ── Public API: apply ───────────────────────────────────────────────

    /// Apply a player action and return the resulting events.
    ///
    /// State mutates in place. Between two `apply` calls there are no
    /// decisions — all intermediate consequences (enemy turn, status ticks,
    /// etc.) are fully resolved.
    pub fn apply(
        &mut self,
        action: Action,
        content: &ContentTables,
    ) -> Result<Vec<GameEvent>, ActionError> {
        let mut events = Vec::new();
        match action {
            Action::PlayCard { hand_index, target } => {
                self.apply_play_card(hand_index, target, content, &mut events)?;
            }
            Action::EndTurn => {
                self.apply_end_turn(content, &mut events)?;
            }
            Action::DiscardCard { hand_index } => {
                self.apply_discard_card(hand_index, &mut events)?;
            }
        }
        Ok(events)
    }

    // ── Public API: legal actions ───────────────────────────────────────

    /// Returns every action the current player/agent can legally take.
    pub fn legal_actions(&self, content: &ContentTables) -> Vec<Action> {
        if matches!(self.phase, CombatPhase::Victory | CombatPhase::Defeat) {
            return vec![];
        }
        if self.phase != CombatPhase::PlayerTurn {
            return vec![];
        }

        let mut actions = Vec::new();

        // Forced discards must be resolved before anything else.
        if self.pending_discards > 0 {
            for i in 0..self.hand.len() {
                actions.push(Action::DiscardCard { hand_index: i });
            }
            return actions;
        }

        // Playable cards (with target expansion for single-target effects).
        for (i, card) in self.hand.iter().enumerate() {
            if self.can_play_card(i, content) {
                if let Some(def) = content.card_defs.get(&card.def_id) {
                    if def.needs_single_target() {
                        let requires_marked = def.effects.iter().any(|e| {
                            matches!(
                                e,
                                Effect::DealDamageIfMarked { .. }
                                    | Effect::DealDamagePerMarkedStack { .. }
                            )
                        });
                        for (j, enemy) in self.enemies.iter().enumerate() {
                            if !enemy.is_dead() {
                                if requires_marked
                                    && enemy.status_effects.get(StatusType::Marked) <= 0
                                {
                                    continue;
                                }
                                actions.push(Action::PlayCard {
                                    hand_index: i,
                                    target: Some(j),
                                });
                            }
                        }
                    } else {
                        actions.push(Action::PlayCard {
                            hand_index: i,
                            target: None,
                        });
                    }
                }
            }
        }

        actions.push(Action::EndTurn);
        actions
    }

    /// Check whether a card at the given hand index is playable.
    pub fn can_play_card(&self, hand_index: usize, content: &ContentTables) -> bool {
        if self.pending_discards > 0 {
            return false;
        }
        let Some(card) = self.hand.get(hand_index) else {
            return false;
        };
        let Some(def) = content.card_defs.get(&card.def_id) else {
            return false;
        };
        if self.player.energy < def.cost {
            return false;
        }
        if def.concentration && self.concentration_active {
            return false;
        }
        true
    }

    // ── Play card ───────────────────────────────────────────────────────

    fn apply_play_card(
        &mut self,
        hand_index: usize,
        target_index: Option<usize>,
        content: &ContentTables,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), ActionError> {
        if matches!(self.phase, CombatPhase::Victory | CombatPhase::Defeat) {
            return Err(ActionError::CombatOver);
        }
        if self.phase != CombatPhase::PlayerTurn {
            return Err(ActionError::NotPlayerTurn);
        }
        if self.pending_discards > 0 {
            return Err(ActionError::MustDiscardFirst);
        }
        if hand_index >= self.hand.len() {
            return Err(ActionError::InvalidHandIndex);
        }

        let def = content
            .card_defs
            .get(&self.hand[hand_index].def_id)
            .expect("card def missing")
            .clone();

        if !self.player.spend_energy(def.cost) {
            return Err(ActionError::InsufficientEnergy);
        }

        // Validate target for single-target effects.
        if def.needs_single_target() {
            match target_index {
                Some(idx) if idx < self.enemies.len() && !self.enemies[idx].is_dead() => {}
                _ => {
                    self.player.energy += def.cost; // refund
                    return Err(ActionError::InvalidTarget);
                }
            }
        }

        let card_inst = self.hand.remove(hand_index);
        let card_id = card_inst.def_id.clone();

        events.push(GameEvent::CardPlayed {
            card_id: card_id.clone(),
            hand_index,
            energy_spent: def.cost,
        });

        self.cards_played_this_turn += 1;

        // Flowing Stance: gain 1 Ki per card played (Monk).
        if self.player.status_effects.has(StatusType::StanceFlowing) {
            self.player.status_effects.apply(StatusType::Ki, 1);
            events.push(GameEvent::StatusApplied {
                target: EventTarget::Player,
                status: StatusType::Ki,
                stacks: 1,
                new_total: self.player.status_effects.get(StatusType::Ki),
            });
        }

        // Arcane Charge (Wizard): Skill cards (non-Attack) grant +1 charge;
        // Attack cards consume ALL charges for a damage bonus.
        let has_attack_tag = def.tags.contains(&CardTag::Attack);
        let has_skill_tag = def.tags.contains(&CardTag::Skill);
        let mut arcane_bonus = 0i32;
        let mut arcane_draw = false;

        if has_attack_tag {
            let charges = self.player.status_effects.get(StatusType::ArcaneCharge);
            if charges > 0 {
                let (bonus, draw) = arcane_charge_bonus(charges);
                arcane_bonus = bonus;
                arcane_draw = draw;
                // Consume all charges.
                self.player.status_effects.apply(StatusType::ArcaneCharge, -charges);
                events.push(GameEvent::StatusApplied {
                    target: EventTarget::Player,
                    status: StatusType::ArcaneCharge,
                    stacks: -charges,
                    new_total: 0,
                });
                // Temporarily boost Empowered for damage calculation.
                if bonus > 0 {
                    self.player.status_effects.apply(StatusType::Empowered, bonus);
                }
            }
        } else if has_skill_tag {
            self.player.status_effects.apply(StatusType::ArcaneCharge, 1);
            events.push(GameEvent::StatusApplied {
                target: EventTarget::Player,
                status: StatusType::ArcaneCharge,
                stacks: 1,
                new_total: self.player.status_effects.get(StatusType::ArcaneCharge),
            });
        }

        let is_consumable = def.card_type == CardType::Consumable;
        let should_exhaust = is_consumable || def.exhaust;

        // Resolve effects — all attacks auto-hit, no delivery resolution needed.
        let mut effects = def.effects.clone();

        // Wild Shape passive bonuses (Druid).
        let is_attack = def.tags.contains(&crate::class::CardTag::Attack);
        let is_defense = def.tags.contains(&crate::class::CardTag::Defense);

        if self.player.status_effects.has(StatusType::WildShapeBear) && is_defense {
            effects.push(Effect::GainBlock { amount: 5 });
        }
        if self.player.status_effects.has(StatusType::WildShapeEagle) && is_attack && !self.eagle_attack_used {
            effects.push(Effect::DrawCards { count: 1 });
            self.eagle_attack_used = true;
        }
        if self.player.status_effects.has(StatusType::WildShapeWolf) && is_attack {
            // +3 damage to all attack damage effects
            for effect in &mut effects {
                match effect {
                    Effect::DealDamage { amount, .. } => *amount += 3,
                    Effect::DealDamageIfInWildShape { base, .. } => *base += 3,
                    Effect::DealDamageWithFormBonus { base, .. } => *base += 3,
                    Effect::DealDamageIfTargetDebuffed { amount, .. } => *amount += 3,
                    _ => {}
                }
            }
        }

        for effect in &effects {
            self.resolve_effect(effect, target_index, events, content);
        }

        // Remove temporary Empowered bonus from Arcane Charge consumption.
        if arcane_bonus > 0 {
            self.player.status_effects.apply(StatusType::Empowered, -arcane_bonus);
        }
        if arcane_draw {
            self.draw_cards(1, events);
        }

        // Card disposition: exhaust > concentration > recycle > discard.
        if should_exhaust {
            events.push(GameEvent::CardExhausted { card_id });
            self.exhaust.push(card_inst);
        } else if def.concentration {
            // Concentration and Wild Shape are mutually exclusive.
            // Playing a concentration card clears any active Wild Shape.
            for ws in &[StatusType::WildShapeBear, StatusType::WildShapeEagle, StatusType::WildShapeWolf] {
                if self.player.status_effects.has(*ws) {
                    self.player.status_effects.apply(*ws, -1);
                }
            }
            let old_conc: Vec<usize> = self
                .hand
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    content
                        .card_defs
                        .get(&c.def_id)
                        .map(|d| d.concentration)
                        .unwrap_or(false)
                })
                .map(|(idx, _)| idx)
                .collect();
            for idx in old_conc.into_iter().rev() {
                let removed = self.hand.remove(idx);
                events.push(GameEvent::CardDiscarded {
                    card_id: removed.def_id.clone(),
                });
                self.discard.push(removed);
            }
            self.push_to_hand_or_discard(card_inst);
            self.concentration_active = true;
        } else if def.recycle {
            self.draw_pile.insert(0, card_inst);
        } else {
            self.discard.push(card_inst);
        }

        self.check_end_conditions(events);
        Ok(())
    }

    // ── End turn ────────────────────────────────────────────────────────

    fn apply_end_turn(
        &mut self,
        content: &ContentTables,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), ActionError> {
        if matches!(self.phase, CombatPhase::Victory | CombatPhase::Defeat) {
            return Err(ActionError::CombatOver);
        }
        if self.phase != CombatPhase::PlayerTurn {
            return Err(ActionError::NotPlayerTurn);
        }
        if self.pending_discards > 0 {
            return Err(ActionError::MustDiscardFirst);
        }

        // Mending heals at end of player turn (before status decay).
        let mending_heal = self.player.status_effects.tick_mending();
        if mending_heal > 0 {
            self.player.heal(mending_heal);
            events.push(GameEvent::Healed {
                target: EventTarget::Player,
                amount: mending_heal,
                new_hp: self.player.hp,
            });
        }

        // Tick player statuses (Bleeding damage, waning decay).
        let tick_result = self.player.status_effects.tick();
        let dot_damage = tick_result.total_damage();
        if dot_damage > 0 {
            self.player.hp -= dot_damage;
            events.push(GameEvent::DotDamage {
                target: EventTarget::Player,
                source: StatusType::Bleeding,
                damage: dot_damage,
            });
            self.check_end_conditions(events);
            if matches!(self.phase, CombatPhase::Victory | CombatPhase::Defeat) {
                return Ok(());
            }
        }

        // Discard hand, retaining the most recent concentration card.
        let mut concentration_card: Option<CardInstance> = None;
        let hand = std::mem::take(&mut self.hand);
        for card in hand.into_iter().rev() {
            let is_conc = content
                .card_defs
                .get(&card.def_id)
                .map(|d| d.concentration)
                .unwrap_or(false);
            if is_conc && concentration_card.is_none() {
                concentration_card = Some(card);
            } else {
                self.discard.push(card);
            }
        }
        if let Some(card) = concentration_card {
            self.push_to_hand_or_discard(card);
        }

        events.push(GameEvent::TurnEnded { turn: self.turn });
        self.phase = CombatPhase::EnemyTurn;
        self.execute_enemy_turn(content, events);

        Ok(())
    }

    // ── Discard card ────────────────────────────────────────────────────

    fn apply_discard_card(
        &mut self,
        hand_index: usize,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), ActionError> {
        if matches!(self.phase, CombatPhase::Victory | CombatPhase::Defeat) {
            return Err(ActionError::CombatOver);
        }
        if self.phase != CombatPhase::PlayerTurn {
            return Err(ActionError::NotPlayerTurn);
        }
        if self.pending_discards == 0 {
            return Err(ActionError::NoPendingDiscards);
        }
        if hand_index >= self.hand.len() {
            return Err(ActionError::InvalidHandIndex);
        }

        let card = self.hand.remove(hand_index);
        events.push(GameEvent::CardDiscarded {
            card_id: card.def_id.clone(),
        });
        self.discard.push(card);
        self.pending_discards -= 1;

        Ok(())
    }

    // ── Draw cards ──────────────────────────────────────────────────────

    fn draw_cards(&mut self, count: usize, events: &mut Vec<GameEvent>) {
        let mut drawn = 0;
        for _ in 0..count {
            if self.hand.len() >= MAX_HAND_SIZE {
                break;
            }
            if self.draw_pile.is_empty() {
                if self.discard.is_empty() {
                    break;
                }
                self.draw_pile.append(&mut self.discard);
                self.rng.shuffle(&mut self.draw_pile);
            }
            if let Some(card) = self.draw_pile.pop() {
                self.hand.push(card);
                drawn += 1;
            }
        }
        if drawn > 0 {
            events.push(GameEvent::CardsDrawn { count: drawn });
        }
    }

    fn push_to_hand_or_discard(&mut self, card: CardInstance) {
        if self.hand.len() < MAX_HAND_SIZE {
            self.hand.push(card);
        } else {
            self.discard.push(card);
        }
    }

    // ── Effect resolution ───────────────────────────────────────────────

    fn resolve_effect(
        &mut self,
        effect: &Effect,
        target_index: Option<usize>,
        events: &mut Vec<GameEvent>,
        content: &ContentTables,
    ) {
        match effect {
            Effect::DealDamage { target, amount } => {
                self.resolve_deal_damage(
                    *target,
                    *amount,
                    target_index,
                    events,
                );
            }
            Effect::GainBlock { amount } => {
                let mut gain = (*amount).max(0);
                // Defensive Stance: +3 block gained.
                if self.player.status_effects.has(StatusType::StanceDefensive) {
                    gain += 3;
                }
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::ApplyStatus {
                target,
                status,
                stacks,
            } => {
                self.resolve_apply_status(*target, *status, *stacks, target_index, events);
            }
            Effect::DrawCards { count } => {
                self.draw_cards(*count, events);
            }
            Effect::GainEnergy { amount } => {
                self.player.energy += amount;
                events.push(GameEvent::EnergyGained { amount: *amount });
            }
            Effect::Heal { amount } => {
                self.player.heal(*amount);
                events.push(GameEvent::Healed {
                    target: EventTarget::Player,
                    amount: *amount,
                    new_hp: self.player.hp,
                });
            }
            Effect::LoseHp { amount } => {
                let old_hp = self.player.hp;
                self.player.hp = (self.player.hp - amount).max(0);
                let hp_lost = old_hp - self.player.hp;
                events.push(GameEvent::DamageDealt {
                    target: EventTarget::Player,
                    base: *amount,
                    modified: *amount,
                    blocked: 0,
                    hp_lost,
                });
                self.apply_hp_loss_triggers(hp_lost, events);
            }
            Effect::DealDamageIfDamagedLastTurn { target, amount } => {
                if self.player_took_damage_last_turn {
                    self.resolve_deal_damage(
                        *target,
                        *amount,
                        target_index,
                        events,
                    );
                }
            }
            Effect::DealDamageIfTargetDebuffed {
                target,
                amount,
                bonus,
            } => {
                let has_debuff = match target {
                    Target::SingleEnemy => target_index
                        .and_then(|idx| self.enemies.get(idx))
                        .map(|e| e.status_effects.has_any_debuff())
                        .unwrap_or(false),
                    _ => false,
                };
                let final_amount = if has_debuff { amount + bonus } else { *amount };
                self.resolve_deal_damage(
                    *target,
                    final_amount,
                    target_index,
                    events,
                );
            }
            Effect::DrawAndDiscard { draw, discard } => {
                self.draw_cards(*draw, events);
                self.pending_discards = (*discard).min(self.hand.len());
            }
            Effect::RemoveEnemyBlock { target, amount } => match target {
                Target::SingleEnemy => {
                    if let Some(idx) = target_index {
                        if idx < self.enemies.len() {
                            self.enemies[idx].block = (self.enemies[idx].block - amount).max(0);
                        }
                    }
                }
                Target::AllEnemies => {
                    for enemy in &mut self.enemies {
                        if !enemy.is_dead() {
                            enemy.block = (enemy.block - amount).max(0);
                        }
                    }
                }
                _ => {}
            },
            Effect::GainBlockIfDamagedLastTurn { amount } => {
                if self.player_took_damage_last_turn {
                    let gain = (*amount).max(0);
                    self.player.gain_block(gain);
                    events.push(GameEvent::BlockGained {
                        target: EventTarget::Player,
                        amount: gain,
                    });
                }
            }
            Effect::BlockEqualMissingHp { max } => {
                let missing = (self.player.max_hp - self.player.hp).max(0);
                let gain = missing.min(*max).max(0);
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::DealDamageIfEnemyBlocked {
                target,
                amount,
                bonus,
            } => {
                let has_block = match target {
                    Target::SingleEnemy => target_index
                        .and_then(|idx| self.enemies.get(idx))
                        .map(|e| e.block > 0)
                        .unwrap_or(false),
                    _ => false,
                };
                let final_amount = if has_block { amount + bonus } else { *amount };
                self.resolve_deal_damage(
                    *target,
                    final_amount,
                    target_index,
                    events,
                );
            }
            Effect::DealDamageEqualBlock { target } => {
                let amount = self.player.block;
                self.resolve_deal_damage(
                    *target,
                    amount,
                    target_index,
                    events,
                );
            }
            Effect::DealDamagePerCardPlayed { target, per_card } => {
                let amount = per_card * self.cards_played_this_turn as i32;
                self.resolve_deal_damage(
                    *target,
                    amount,
                    target_index,
                    events,
                );
            }
            Effect::DealDamageIfMarked { target, amount } => {
                let has_marked = match target {
                    Target::SingleEnemy => target_index
                        .and_then(|idx| self.enemies.get(idx))
                        .map(|e| e.status_effects.get(StatusType::Marked) > 0)
                        .unwrap_or(false),
                    _ => false,
                };
                if has_marked {
                    self.resolve_deal_damage(
                        *target,
                        *amount,
                        target_index,
                        events,
                    );
                }
            }
            Effect::ApplyStatusPerCardPlayed {
                target,
                status,
                per_card,
            } => {
                let total_stacks = per_card * self.cards_played_this_turn as i32;
                self.resolve_apply_status(*target, *status, total_stacks, target_index, events);
            }
            Effect::DealDamagePerMarkedStack { target, per_stack } => {
                if target == &Target::SingleEnemy {
                    if let Some(idx) = target_index {
                        if idx < self.enemies.len() && !self.enemies[idx].is_dead() {
                            let stacks = self.enemies[idx].status_effects.get(StatusType::Marked);
                            if stacks > 0 {
                                let amount = per_stack * stacks;
                                self.resolve_deal_damage(
                                    *target,
                                    amount,
                                    target_index,
                                    events,
                                );
                                // Remove all Marked stacks.
                                self.enemies[idx]
                                    .status_effects
                                    .apply(StatusType::Marked, -stacks);
                            }
                        }
                    }
                }
            }
            Effect::GainBlockPerStatusStack { status, multiplier } => {
                let stacks = self.player.status_effects.get(*status);
                let gain = (stacks * multiplier).max(0);
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::DrawPerEnemyWithStatus { status } => {
                let count = self
                    .enemies
                    .iter()
                    .filter(|e| !e.is_dead() && e.status_effects.has(*status))
                    .count();
                if count > 0 {
                    self.draw_cards(count, events);
                }
            }
            // ── New effect variants ─────────────────────────────────────
            Effect::GainBlockConditional {
                amount,
                bonus_amount,
            } => {
                let gain = if self.player.block > 0 {
                    *bonus_amount
                } else {
                    *amount
                };
                let gain = gain.max(0);
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::BlockFloor { amount } => {
                self.block_floor = *amount;
            }
            Effect::RetainBlockPartial { amount } => {
                self.retain_block_cap = *amount;
            }
            Effect::DoubleStatus { target, status } => match target {
                Target::SingleEnemy => {
                    if let Some(idx) = target_index {
                        if idx < self.enemies.len() && !self.enemies[idx].is_dead() {
                            let current = self.enemies[idx].status_effects.get(*status);
                            if current > 0 {
                                self.enemies[idx].status_effects.apply(*status, current);
                                events.push(GameEvent::StatusApplied {
                                    target: EventTarget::Enemy(idx),
                                    status: *status,
                                    stacks: current,
                                    new_total: self.enemies[idx].status_effects.get(*status),
                                });
                            }
                        }
                    }
                }
                Target::AllEnemies => {
                    for i in 0..self.enemies.len() {
                        if !self.enemies[i].is_dead() {
                            let current = self.enemies[i].status_effects.get(*status);
                            if current > 0 {
                                self.enemies[i].status_effects.apply(*status, current);
                                events.push(GameEvent::StatusApplied {
                                    target: EventTarget::Enemy(i),
                                    status: *status,
                                    stacks: current,
                                    new_total: self.enemies[i].status_effects.get(*status),
                                });
                            }
                        }
                    }
                }
                Target::Player => {
                    let current = self.player.status_effects.get(*status);
                    if current > 0 {
                        self.player.status_effects.apply(*status, current);
                        events.push(GameEvent::StatusApplied {
                            target: EventTarget::Player,
                            status: *status,
                            stacks: current,
                            new_total: self.player.status_effects.get(*status),
                        });
                    }
                }
                Target::RandomEnemy => {}
            },
            Effect::DealDamageScaledByMarked {
                target,
                base_damage,
                per_stack,
            } => {
                if target == &Target::SingleEnemy {
                    if let Some(idx) = target_index {
                        if idx < self.enemies.len() && !self.enemies[idx].is_dead() {
                            let stacks = self.enemies[idx].status_effects.get(StatusType::Marked);
                            let amount = base_damage + per_stack * stacks;
                            self.resolve_deal_damage(
                                *target,
                                amount,
                                target_index,
                                events,
                            );
                        }
                    }
                }
            }
            Effect::EnterStance { stance } => {
                // Remove any existing stance before entering the new one.
                for s in &[StatusType::StanceAggressive, StatusType::StanceDefensive, StatusType::StanceFlowing] {
                    if self.player.status_effects.has(*s) {
                        self.player.status_effects.apply(*s, -1);
                    }
                }
                self.player.status_effects.apply(*stance, 1);
                events.push(GameEvent::StatusApplied {
                    target: EventTarget::Player,
                    status: *stance,
                    stacks: 1,
                    new_total: 1,
                });
            }
            Effect::DealDamageSpendKi { target, base, ki_cost, bonus } => {
                let ki = self.player.status_effects.get(StatusType::Ki);
                let amount = if ki >= *ki_cost {
                    self.player.status_effects.apply(StatusType::Ki, -ki_cost);
                    *bonus
                } else {
                    *base
                };
                self.resolve_deal_damage(*target, amount, target_index, events);
            }
            Effect::DealDamageSpendArtifice { target, base, artifice_cost, bonus } => {
                let art = self.player.status_effects.get(StatusType::Artifice);
                let amount = if art >= *artifice_cost {
                    self.player.status_effects.apply(StatusType::Artifice, -artifice_cost);
                    *bonus
                } else {
                    *base
                };
                self.resolve_deal_damage(*target, amount, target_index, events);
            }
            Effect::GainBlockSpendKi { base, ki_cost, bonus } => {
                let ki = self.player.status_effects.get(StatusType::Ki);
                let mut gain = if ki >= *ki_cost {
                    self.player.status_effects.apply(StatusType::Ki, -ki_cost);
                    *bonus
                } else {
                    *base
                };
                if self.player.status_effects.has(StatusType::StanceDefensive) {
                    gain += 3;
                }
                let gain = gain.max(0);
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::GainBlockPerKiStack { multiplier } => {
                let ki = self.player.status_effects.get(StatusType::Ki);
                let mut gain = (ki * multiplier).max(0);
                if self.player.status_effects.has(StatusType::StanceDefensive) {
                    gain += 3;
                }
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::DealDamageSpendSmite { target, base, smite_cost, bonus } => {
                let smite = self.player.status_effects.get(StatusType::Smite);
                let amount = if smite >= *smite_cost {
                    self.player.status_effects.apply(StatusType::Smite, -smite_cost);
                    *bonus
                } else {
                    *base
                };
                self.resolve_deal_damage(*target, amount, target_index, events);
            }
            Effect::GainBlockSpendSmite { base, smite_cost, bonus } => {
                let smite = self.player.status_effects.get(StatusType::Smite);
                let gain = if smite >= *smite_cost {
                    self.player.status_effects.apply(StatusType::Smite, -smite_cost);
                    *bonus
                } else {
                    *base
                };
                let gain = gain.max(0);
                self.player.gain_block(gain);
                events.push(GameEvent::BlockGained {
                    target: EventTarget::Player,
                    amount: gain,
                });
            }
            Effect::DealDamagePerPlayerStatus {
                target,
                status,
                per_stack,
            } => {
                let stacks = self.player.status_effects.get(*status);
                let amount = per_stack * stacks;
                if amount > 0 {
                    self.resolve_deal_damage(*target, amount, target_index, events);
                }
            }
            Effect::DealDamagePerDebuffOnTarget {
                target,
                base,
                per_debuff,
            } => {
                let calc_amount = |enemy: &crate::enemy::EnemyState| -> i32 {
                    let debuff_count = enemy.status_effects.debuff_count();
                    base + per_debuff * debuff_count
                };
                match target {
                    Target::SingleEnemy => {
                        if let Some(idx) = target_index {
                            if idx < self.enemies.len() && !self.enemies[idx].is_dead() {
                                let amount = calc_amount(&self.enemies[idx]);
                                self.resolve_deal_damage(*target, amount, target_index, events);
                            }
                        }
                    }
                    Target::AllEnemies => {
                        for i in 0..self.enemies.len() {
                            if !self.enemies[i].is_dead() {
                                let amount = calc_amount(&self.enemies[i]);
                                self.resolve_deal_damage(*target, amount, Some(i), events);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Effect::DealDamageScaledByMissingHp {
                target,
                base,
                per_5_missing,
                cap,
            } => {
                let missing = (self.player.max_hp - self.player.hp).max(0);
                let bonus = ((missing / 5) * per_5_missing).min(*cap);
                let amount = base + bonus;
                self.resolve_deal_damage(*target, amount, target_index, events);
            }
            Effect::DealDamageScaledByMissingHpEnemy {
                target,
                base,
                per_10_percent_missing,
            } => {
                match target {
                    Target::SingleEnemy => {
                        if let Some(idx) = target_index {
                            if idx < self.enemies.len() && !self.enemies[idx].is_dead() {
                                let max_hp = self.enemies[idx].max_hp.max(1);
                                let missing_pct = ((max_hp - self.enemies[idx].hp) * 100) / max_hp;
                                let bonus = (missing_pct / 10) * per_10_percent_missing;
                                let amount = base + bonus;
                                self.resolve_deal_damage(*target, amount, target_index, events);
                            }
                        }
                    }
                    Target::AllEnemies => {
                        for i in 0..self.enemies.len() {
                            if !self.enemies[i].is_dead() {
                                let max_hp = self.enemies[i].max_hp.max(1);
                                let missing_pct = ((max_hp - self.enemies[i].hp) * 100) / max_hp;
                                let bonus = (missing_pct / 10) * per_10_percent_missing;
                                let amount = base + bonus;
                                self.resolve_deal_damage(*target, amount, Some(i), events);
                            }
                        }
                    }
                    _ => {}
                }
            }
            // ── Druid Wild Shape effects ─────────────────────────────────
            Effect::EnterWildShape { form } => {
                // Clear any existing Wild Shape.
                for ws in &[StatusType::WildShapeBear, StatusType::WildShapeEagle, StatusType::WildShapeWolf] {
                    if self.player.status_effects.has(*ws) {
                        self.player.status_effects.apply(*ws, -1);
                    }
                }
                // Clear concentration (discard concentration cards from hand).
                if self.concentration_active {
                    let conc_indices: Vec<usize> = self
                        .hand
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| {
                            content.card_defs.get(&c.def_id)
                                .map(|d| d.concentration)
                                .unwrap_or(false)
                        })
                        .map(|(idx, _)| idx)
                        .collect();
                    for idx in conc_indices.into_iter().rev() {
                        let removed = self.hand.remove(idx);
                        events.push(GameEvent::CardDiscarded {
                            card_id: removed.def_id.clone(),
                        });
                        self.discard.push(removed);
                    }
                    self.concentration_active = false;
                }
                // Apply the new form.
                self.player.status_effects.apply(*form, 1);
                events.push(GameEvent::StatusApplied {
                    target: EventTarget::Player,
                    status: *form,
                    stacks: 1,
                    new_total: 1,
                });
            }
            Effect::DealDamageIfInWildShape { target, base, bonus } => {
                let in_wild_shape = self.player.status_effects.has(StatusType::WildShapeBear)
                    || self.player.status_effects.has(StatusType::WildShapeEagle)
                    || self.player.status_effects.has(StatusType::WildShapeWolf);
                let amount = if in_wild_shape { base + bonus } else { *base };
                self.resolve_deal_damage(*target, amount, target_index, events);
            }
            Effect::DealDamageWithFormBonus { target, base, status, bonus } => {
                let amount = if self.player.status_effects.has(*status) {
                    base + bonus
                } else {
                    *base
                };
                self.resolve_deal_damage(*target, amount, target_index, events);
            }
            Effect::GainBlockWithFormBonus { base, status, bonus } => {
                let mut gain = if self.player.status_effects.has(*status) {
                    base + bonus
                } else {
                    *base
                };
                if self.player.status_effects.has(StatusType::StanceDefensive) {
                    gain += 3;
                }
                let gain = gain.max(0);
                if gain > 0 {
                    self.player.gain_block(gain);
                    events.push(GameEvent::BlockGained {
                        target: EventTarget::Player,
                        amount: gain,
                    });
                }
            }
            Effect::DrawCardsWithFormBonus { base_draw, status, bonus_draw } => {
                let count = if self.player.status_effects.has(*status) {
                    base_draw + bonus_draw
                } else {
                    *base_draw
                };
                if count > 0 {
                    self.draw_cards(count, events);
                }
            }
            Effect::DealDamageIfPlayerHasStatus { target, status, amount } => {
                if self.player.status_effects.has(*status) {
                    self.resolve_deal_damage(*target, *amount, target_index, events);
                }
            }
            Effect::ApplyStatusIfPlayerHasStatus {
                status,
                stacks,
                require_status,
                require_stacks,
            } => {
                if self.player.status_effects.get(*require_status) >= *require_stacks {
                    self.player.status_effects.apply(*status, *stacks);
                    events.push(GameEvent::StatusApplied {
                        target: EventTarget::Player,
                        status: *status,
                        stacks: *stacks,
                        new_total: self.player.status_effects.get(*status),
                    });
                }
            }
        }
    }

    // ── Damage resolution ───────────────────────────────────────────────

    fn resolve_deal_damage(
        &mut self,
        target: Target,
        amount: i32,
        target_index: Option<usize>,
        events: &mut Vec<GameEvent>,
    ) {
        let attacker_statuses = self.player.status_effects.clone();
        match target {
            Target::SingleEnemy => {
                if let Some(idx) = target_index {
                    if idx < self.enemies.len() && !self.enemies[idx].is_dead() {
                        let threatened =
                            self.enemies[idx].status_effects.has(StatusType::Threatened);
                        let marked = self.enemies[idx].status_effects.get(StatusType::Marked);
                        let rolled = amount;
                        let single_target = true;
                        let modified = Self::calculate_damage(
                            rolled,
                            &attacker_statuses,
                            threatened,
                            marked,
                            single_target,
                        );
                        let hp_lost = self.enemies[idx].take_damage(modified);
                        events.push(GameEvent::DamageDealt {
                            target: EventTarget::Enemy(idx),
                            base: amount,
                            modified,
                            blocked: modified - hp_lost,
                            hp_lost,
                        });
                        // Momentum: draw 1 card if player has Momentum and dealt 12+ damage.
                        if self.player.status_effects.has(StatusType::Momentum) && modified >= 12 {
                            self.draw_cards(1, events);
                        }
                        if self.enemies[idx].is_dead() {
                            events.push(GameEvent::EnemyDied { enemy_index: idx });
                        }
                    }
                }
            }
            Target::AllEnemies => {
                for i in 0..self.enemies.len() {
                    if !self.enemies[i].is_dead() {
                        let threatened = self.enemies[i].status_effects.has(StatusType::Threatened);
                        let marked = self.enemies[i].status_effects.get(StatusType::Marked);
                        let rolled = amount;
                        let single_target = false;
                        let modified = Self::calculate_damage(
                            rolled,
                            &attacker_statuses,
                            threatened,
                            marked,
                            single_target,
                        );
                        let hp_lost = self.enemies[i].take_damage(modified);
                        events.push(GameEvent::DamageDealt {
                            target: EventTarget::Enemy(i),
                            base: amount,
                            modified,
                            blocked: modified - hp_lost,
                            hp_lost,
                        });
                        if self.enemies[i].is_dead() {
                            events.push(GameEvent::EnemyDied { enemy_index: i });
                        }
                    }
                }
            }
            Target::RandomEnemy => {
                let alive: Vec<usize> = self
                    .enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| !e.is_dead())
                    .map(|(i, _)| i)
                    .collect();
                if !alive.is_empty() {
                    let pick = self.rng.range(0, (alive.len() as i32) - 1) as usize;
                    let idx = alive[pick];
                    let threatened = self.enemies[idx].status_effects.has(StatusType::Threatened);
                    let marked = self.enemies[idx].status_effects.get(StatusType::Marked);
                    let rolled = amount;
                    let single_target = true;
                    let modified = Self::calculate_damage(
                        rolled,
                        &attacker_statuses,
                        threatened,
                        marked,
                        single_target,
                    );
                    let hp_lost = self.enemies[idx].take_damage(modified);
                    events.push(GameEvent::DamageDealt {
                        target: EventTarget::Enemy(idx),
                        base: amount,
                        modified,
                        blocked: modified - hp_lost,
                        hp_lost,
                    });
                    // Momentum: draw 1 card if player has Momentum and dealt 12+ damage.
                    if self.player.status_effects.has(StatusType::Momentum) && modified >= 12 {
                        self.draw_cards(1, events);
                    }
                    if self.enemies[idx].is_dead() {
                        events.push(GameEvent::EnemyDied { enemy_index: idx });
                    }
                }
            }
            Target::Player => {
                let threatened = self.player.status_effects.has(StatusType::Threatened);
                let marked = self.player.status_effects.get(StatusType::Marked);
                let modified =
                    Self::calculate_damage(amount, &attacker_statuses, threatened, marked, false);
                let hp_lost = self.player.take_damage(modified);
                events.push(GameEvent::DamageDealt {
                    target: EventTarget::Player,
                    base: amount,
                    modified,
                    blocked: modified - hp_lost,
                    hp_lost,
                });
            }
        }
    }

    fn resolve_apply_status(
        &mut self,
        target: Target,
        status: StatusType,
        stacks: i32,
        target_index: Option<usize>,
        events: &mut Vec<GameEvent>,
    ) {
        match target {
            Target::SingleEnemy => {
                if let Some(idx) = target_index {
                    if idx < self.enemies.len() {
                        self.enemies[idx].status_effects.apply(status, stacks);
                        events.push(GameEvent::StatusApplied {
                            target: EventTarget::Enemy(idx),
                            status,
                            stacks,
                            new_total: self.enemies[idx].status_effects.get(status),
                        });
                    }
                }
            }
            Target::AllEnemies => {
                for i in 0..self.enemies.len() {
                    if !self.enemies[i].is_dead() {
                        self.enemies[i].status_effects.apply(status, stacks);
                        events.push(GameEvent::StatusApplied {
                            target: EventTarget::Enemy(i),
                            status,
                            stacks,
                            new_total: self.enemies[i].status_effects.get(status),
                        });
                    }
                }
            }
            Target::Player => {
                self.player.status_effects.apply(status, stacks);
                // Artifice is capped at 20.
                if status == StatusType::Artifice {
                    let current = self.player.status_effects.get(StatusType::Artifice);
                    if current > 20 {
                        self.player.status_effects.apply(StatusType::Artifice, 20 - current);
                    }
                }
                events.push(GameEvent::StatusApplied {
                    target: EventTarget::Player,
                    status,
                    stacks,
                    new_total: self.player.status_effects.get(status),
                });
            }
            Target::RandomEnemy => {
                let alive: Vec<usize> = self
                    .enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| !e.is_dead())
                    .map(|(i, _)| i)
                    .collect();
                if !alive.is_empty() {
                    let pick = self.rng.range(0, (alive.len() as i32) - 1) as usize;
                    let idx = alive[pick];
                    self.enemies[idx].status_effects.apply(status, stacks);
                    events.push(GameEvent::StatusApplied {
                        target: EventTarget::Enemy(idx),
                        status,
                        stacks,
                        new_total: self.enemies[idx].status_effects.get(status),
                    });
                }
            }
        }
    }

    // ── HP-loss triggers (Barbarian) ────────────────────────────────────

    /// Check and apply PainIsPower (HP loss → Rage) and WardingTotem (HP loss → block).
    fn apply_hp_loss_triggers(&mut self, hp_lost: i32, events: &mut Vec<GameEvent>) {
        if hp_lost <= 0 {
            return;
        }
        if self.player.status_effects.has(StatusType::PainIsPower) {
            self.player.status_effects.apply(StatusType::Rage, hp_lost);
            events.push(GameEvent::StatusApplied {
                target: EventTarget::Player,
                status: StatusType::Rage,
                stacks: hp_lost,
                new_total: self.player.status_effects.get(StatusType::Rage),
            });
        }
        if self.player.status_effects.has(StatusType::WardingTotem) {
            let block_gain = 3;
            self.player.gain_block(block_gain);
            events.push(GameEvent::BlockGained {
                target: EventTarget::Player,
                amount: block_gain,
            });
        }
    }

    // ── Damage pipeline ─────────────────────────────────────────────────

    /// Calculate final damage with Empowered, SavageBlows, Weakened, Threatened, and Marked.
    ///
    /// Pipeline: base + Empowered [+ SavageBlows if single_target] → ×0.75 if Weakened
    /// → ×1.5 if Threatened → + Marked.
    /// A miss (base ≤ 0) short-circuits to zero.
    fn calculate_damage(
        base: i32,
        attacker_statuses: &StatusMap,
        defender_threatened: bool,
        defender_marked: i32,
        single_target: bool,
    ) -> i32 {
        if base <= 0 {
            return 0;
        }
        let mut effective = base + attacker_statuses.get(StatusType::Empowered)
            + attacker_statuses.get(StatusType::Rage);

        // Stance modifiers (Monk).
        if attacker_statuses.has(StatusType::StanceAggressive) {
            effective += 3;
        }
        if attacker_statuses.has(StatusType::StanceDefensive) {
            effective -= 3;
        }

        if single_target {
            effective += attacker_statuses.get(StatusType::SavageBlows);
        }

        if attacker_statuses.has(StatusType::Weakened) {
            effective = (effective as f64 * 0.75).floor() as i32;
        }
        if defender_threatened {
            effective = (effective as f64 * 1.5).floor() as i32;
        }

        effective += defender_marked;
        effective.max(0)
    }

    // ── Enemy turn ──────────────────────────────────────────────────────

    fn execute_enemy_turn(&mut self, content: &ContentTables, events: &mut Vec<GameEvent>) {
        let enemy_count = self.enemies.len();
        let mut took_damage = false;

        for i in 0..enemy_count {
            if self.enemies[i].is_dead() {
                continue;
            }

            // Cache statuses BEFORE tick — debuffs applied by the player last
            // through the enemy's action; tick decays them afterward.
            let pre_tick_statuses = self.enemies[i].status_effects.clone();
            let is_frightened = pre_tick_statuses.has(StatusType::Frightened);

            // Tick enemy statuses.
            let tick_result = self.enemies[i].status_effects.tick();
            let dot_damage = tick_result.total_damage();
            if dot_damage > 0 {
                self.enemies[i].hp -= dot_damage;
                events.push(GameEvent::DotDamage {
                    target: EventTarget::Enemy(i),
                    source: StatusType::Bleeding,
                    damage: dot_damage,
                });
                if self.enemies[i].is_dead() {
                    events.push(GameEvent::EnemyDied { enemy_index: i });
                    continue;
                }
            }

            // Reset enemy block.
            self.enemies[i].reset_block();

            let intent = match self.enemies[i].current_intent.clone() {
                Some(intent) => intent,
                None => continue,
            };

            // Frightened: skip action but advance intent.
            if is_frightened {
                events.push(GameEvent::EnemySkipped { enemy_index: i });
                let def_id = self.enemies[i].def_id.clone();
                if let Some(def) = self.scaled_enemy_def(content, def_id) {
                    self.enemies[i].advance_intent(&def);
                }
                continue;
            }

            events.push(GameEvent::EnemyAction {
                enemy_index: i,
                intent: intent.clone(),
            });

            let player_threatened = self.player.status_effects.has(StatusType::Threatened);
            let player_marked = self.player.status_effects.get(StatusType::Marked);
            let player_barbed = self.player.status_effects.get(StatusType::Barbed);
            let player_fortified = self.player.status_effects.get(StatusType::Fortified);

            match intent {
                IntentType::Attack(base) => {
                    // Sanctified Shield: negate the first enemy attack.
                    if self.player.status_effects.has(StatusType::SanctifiedShield) {
                        self.player.status_effects.apply(StatusType::SanctifiedShield, -1);
                        events.push(GameEvent::StatusApplied {
                            target: EventTarget::Player,
                            status: StatusType::SanctifiedShield,
                            stacks: -1,
                            new_total: 0,
                        });
                    } else {
                        let mut dmg = Self::calculate_damage(
                            base,
                            &pre_tick_statuses,
                            player_threatened,
                            player_marked,
                            false,
                        );
                        // Aggressive Stance: player takes +3 damage.
                        if self.player.status_effects.has(StatusType::StanceAggressive) {
                            dmg += 3;
                        }
                        // Fortified: reduce incoming damage by stacks.
                        dmg = (dmg - player_fortified).max(0);
                        let hp_lost = self.player.take_damage(dmg);
                        events.push(GameEvent::DamageDealt {
                            target: EventTarget::Player,
                            base,
                            modified: dmg,
                            blocked: dmg - hp_lost,
                            hp_lost,
                        });
                        if hp_lost > 0 {
                            took_damage = true;
                            self.apply_hp_loss_triggers(hp_lost, events);
                        }
                        // Barbed: reflect damage back to attacker
                        if player_barbed > 0 {
                            let reflected = self.enemies[i].take_damage(player_barbed);
                            events.push(GameEvent::DamageDealt {
                                target: EventTarget::Enemy(i),
                                base: player_barbed,
                                modified: player_barbed,
                                blocked: player_barbed - reflected,
                                hp_lost: reflected,
                            });
                            if self.enemies[i].is_dead() {
                                events.push(GameEvent::EnemyDied { enemy_index: i });
                            }
                        }
                    }
                }
                IntentType::Defend(amount) => {
                    self.enemies[i].block += amount;
                    events.push(GameEvent::BlockGained {
                        target: EventTarget::Enemy(i),
                        amount,
                    });
                }
                IntentType::Buff(status, stacks) => {
                    self.enemies[i].status_effects.apply(status, stacks);
                    events.push(GameEvent::StatusApplied {
                        target: EventTarget::Enemy(i),
                        status,
                        stacks,
                        new_total: self.enemies[i].status_effects.get(status),
                    });
                }
                IntentType::Debuff(status, stacks) => {
                    self.player.status_effects.apply(status, stacks);
                    events.push(GameEvent::StatusApplied {
                        target: EventTarget::Player,
                        status,
                        stacks,
                        new_total: self.player.status_effects.get(status),
                    });
                }
                IntentType::AttackDefend(atk, def) => {
                    // Sanctified Shield: negate the attack portion.
                    if self.player.status_effects.has(StatusType::SanctifiedShield) {
                        self.player.status_effects.apply(StatusType::SanctifiedShield, -1);
                        events.push(GameEvent::StatusApplied {
                            target: EventTarget::Player,
                            status: StatusType::SanctifiedShield,
                            stacks: -1,
                            new_total: 0,
                        });
                    } else {
                        let mut dmg = Self::calculate_damage(
                            atk,
                            &pre_tick_statuses,
                            player_threatened,
                            player_marked,
                            false,
                        );
                        if self.player.status_effects.has(StatusType::StanceAggressive) {
                            dmg += 3;
                        }
                        // Fortified: reduce incoming damage by stacks.
                        dmg = (dmg - player_fortified).max(0);
                        let hp_lost = self.player.take_damage(dmg);
                        events.push(GameEvent::DamageDealt {
                            target: EventTarget::Player,
                            base: atk,
                            modified: dmg,
                            blocked: dmg - hp_lost,
                            hp_lost,
                        });
                        if hp_lost > 0 {
                            took_damage = true;
                            self.apply_hp_loss_triggers(hp_lost, events);
                        }
                        // Barbed: reflect damage back to attacker
                        if player_barbed > 0 {
                            let reflected = self.enemies[i].take_damage(player_barbed);
                            events.push(GameEvent::DamageDealt {
                                target: EventTarget::Enemy(i),
                                base: player_barbed,
                                modified: player_barbed,
                                blocked: player_barbed - reflected,
                                hp_lost: reflected,
                            });
                            if self.enemies[i].is_dead() {
                                events.push(GameEvent::EnemyDied { enemy_index: i });
                            }
                        }
                    }
                    // Defend portion still applies even if attack was negated.
                    self.enemies[i].block += def;
                    events.push(GameEvent::BlockGained {
                        target: EventTarget::Enemy(i),
                        amount: def,
                    });
                }
                IntentType::BuffAllies(status, stacks) => {
                    for j in 0..self.enemies.len() {
                        if j != i && !self.enemies[j].is_dead() {
                            self.enemies[j].status_effects.apply(status, stacks);
                            events.push(GameEvent::StatusApplied {
                                target: EventTarget::Enemy(j),
                                status,
                                stacks,
                                new_total: self.enemies[j].status_effects.get(status),
                            });
                        }
                    }
                }
            }

            // Advance intent.
            let def_id = self.enemies[i].def_id.clone();
            if let Some(def) = self.scaled_enemy_def(content, def_id) {
                self.enemies[i].advance_intent(&def);
            }
        }

        self.player_took_damage_last_turn = took_damage;

        self.check_end_conditions(events);
        if matches!(self.phase, CombatPhase::Victory | CombatPhase::Defeat) {
            return;
        }

        // Start new player turn.
        self.turn += 1;
        if self.turn > MAX_TURNS {
            self.phase = CombatPhase::Defeat;
            events.push(GameEvent::CombatDefeat);
            return;
        }
        self.phase = CombatPhase::PlayerTurn;
        if !self.player.status_effects.has(StatusType::BlockRetention) {
            if self.retain_block_cap > 0 {
                self.player.block = self.player.block.min(self.retain_block_cap);
            } else {
                self.player.block = self.block_floor;
            }
        }
        self.player.energy = self.player.max_energy;
        self.cards_played_this_turn = 0;
        self.eagle_attack_used = false;

        // BerserkersTrance: lose 2 HP, draw 1 extra card at start of turn.
        let trance_draw = if self.player.status_effects.has(StatusType::BerserkersTrance) {
            let old_hp = self.player.hp;
            self.player.hp = (self.player.hp - 2).max(0);
            let hp_lost = old_hp - self.player.hp;
            if hp_lost > 0 {
                events.push(GameEvent::DamageDealt {
                    target: EventTarget::Player,
                    base: 2,
                    modified: 2,
                    blocked: 0,
                    hp_lost,
                });
                self.apply_hp_loss_triggers(hp_lost, events);
            }
            1
        } else {
            0
        };

        events.push(GameEvent::TurnStarted { turn: self.turn });
        self.draw_cards(self.draw_per_turn + trance_draw, events);
    }

    fn scaled_enemy_def(&self, content: &ContentTables, id: EnemyId) -> Option<EnemyDef> {
        let base = content.enemy_defs.get(&id)?;
        let scaled = match self.enemy_scaling {
            EnemyScaling::Act { act, boss } => {
                if boss {
                    crate::enemy::scale_boss_def(base, act)
                } else {
                    crate::enemy::scale_enemy_def(base, act)
                }
            }
            EnemyScaling::Custom { hp_mult, dmg_mult } => {
                crate::enemy::scale_enemy_def_custom(base, hp_mult, dmg_mult)
            }
        };
        Some(scaled)
    }

    // ── End conditions ──────────────────────────────────────────────────

    fn check_end_conditions(&mut self, events: &mut Vec<GameEvent>) {
        if self.player.is_dead() {
            self.phase = CombatPhase::Defeat;
            events.push(GameEvent::PlayerDied);
            events.push(GameEvent::CombatDefeat);
            return;
        }
        if self.enemies.iter().all(|e| e.is_dead()) {
            self.phase = CombatPhase::Victory;
            events.push(GameEvent::CombatVictory);
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::*;
    use crate::class::CardTag;

    // ── Test helpers ────────────────────────────────────────────────────

    fn strike_def() -> CardDef {
        CardDef {
            id: "strike".into(),
            name: "Strike".to_string(),
            rarity: Rarity::Common,
            cost: 1,
            card_type: CardType::Spell,
            exhaust: false,
            effects: vec![Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            }],
            description: "Deal 10 damage.".to_string(),
            class: None,
            subclass: None,
            tags: vec![CardTag::Attack],
            recycle: false,
            concentration: false,
            innate: false,
            milestone: false,
            inherent: false,
        }
    }

    fn defend_card_def() -> CardDef {
        CardDef {
            id: "defend".into(),
            name: "Defend".to_string(),
            rarity: Rarity::Common,
            cost: 1,
            card_type: CardType::Spell,
            exhaust: false,
            effects: vec![Effect::GainBlock { amount: 10 }],
            description: "Gain 10 block.".to_string(),
            class: None,
            subclass: None,
            tags: vec![CardTag::Defense],
            recycle: false,
            concentration: false,
            innate: false,
            milestone: false,
            inherent: false,
        }
    }



    fn test_enemy_def() -> crate::enemy::EnemyDef {
        crate::enemy::EnemyDef {
            id: "test_goblin".to_string(),
            name: "Test Goblin".to_string(),
            max_hp: 20,
            intent_pattern: vec![IntentType::Attack(6), IntentType::Defend(4)],
        }
    }

    fn make_content(cards: Vec<CardDef>, enemies: Vec<crate::enemy::EnemyDef>) -> ContentTables {
        ContentTables {
            card_defs: cards.into_iter().map(|d| (d.id.clone(), d)).collect(),
            enemy_defs: enemies.into_iter().map(|d| (d.id.clone(), d)).collect(),
        }
    }

    fn make_combat(content: &ContentTables, deck_ids: &[&str], enemy_ids: &[&str]) -> CombatState {
        let deck: Vec<CardInstance> = deck_ids
            .iter()
            .map(|id| CardInstance {
                def_id: id.to_string(),
                upgraded: false,
            })
            .collect();
        let enemies: Vec<EnemyState> = enemy_ids
            .iter()
            .map(|id| {
                let def = content.enemy_defs.get(*id).expect("enemy def missing");
                EnemyState::from_def(def)
            })
            .collect();
        let player = PlayerState::new(50, 3);
        CombatState::new(
            player,
            deck,
            enemies,
            42,
            0,
            0,
            Some(Class::Fighter),
            content,
        )
    }

    // ── Card play tests ─────────────────────────────────────────────────

    #[test]
    fn play_card_deals_damage() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        assert_eq!(combat.hand.len(), 5);

        let events = combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::CardPlayed { .. })));
        assert!(events.iter().any(|e| matches!(
            e, GameEvent::DamageDealt { target: EventTarget::Enemy(0), hp_lost, .. } if *hp_lost == 10
        )));
        assert_eq!(combat.enemies[0].hp, 10);
        assert_eq!(combat.hand.len(), 4);
        assert_eq!(combat.player.energy, 2);
    }

    #[test]
    fn play_defend_gains_block() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 10], &["test_goblin"]);

        let events = combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();

        assert!(events.iter().any(|e| matches!(
            e,
            GameEvent::BlockGained {
                target: EventTarget::Player,
                amount: 10
            }
        )));
        assert_eq!(combat.player.block, 10);
    }

    #[test]
    fn insufficient_energy_rejected() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.player.energy = 0;

        let result = combat.apply(
            Action::PlayCard {
                hand_index: 0,
                target: Some(0),
            },
            &content,
        );
        assert_eq!(result, Err(ActionError::InsufficientEnergy));
    }

    #[test]
    fn invalid_target_rejected() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);

        let result = combat.apply(
            Action::PlayCard {
                hand_index: 0,
                target: Some(5),
            },
            &content,
        );
        assert_eq!(result, Err(ActionError::InvalidTarget));
        assert_eq!(combat.player.energy, 3); // energy refunded
    }

    // ── End turn tests ──────────────────────────────────────────────────

    #[test]
    fn end_turn_triggers_enemy_and_new_turn() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 20], &["test_goblin"]);

        let events = combat.apply(Action::EndTurn, &content).unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::TurnEnded { turn: 1 })));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::EnemyAction { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::TurnStarted { turn: 2 })));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::CardsDrawn { .. })));
        assert_eq!(combat.turn, 2);
        assert_eq!(combat.phase, CombatPhase::PlayerTurn);
        assert_eq!(combat.player.energy, combat.player.max_energy);
    }

    // ── Victory / defeat ────────────────────────────────────────────────

    #[test]
    fn victory_when_all_enemies_die() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.enemies[0].hp = 5;

        let events = combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::EnemyDied { enemy_index: 0 })));
        assert!(events.iter().any(|e| matches!(e, GameEvent::CombatVictory)));
        assert_eq!(combat.phase, CombatPhase::Victory);
    }

    #[test]
    fn no_actions_after_combat_ends() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.phase = CombatPhase::Victory;

        assert!(combat.legal_actions(&content).is_empty());
        assert_eq!(
            combat.apply(Action::EndTurn, &content),
            Err(ActionError::CombatOver)
        );
    }

    // ── Exhaust / concentration / recycle ────────────────────────────────

    #[test]
    fn exhaust_card_goes_to_exhaust_pile() {
        let mut def = strike_def();
        def.id = "exhaust_strike".into();
        def.exhaust = true;
        let content = make_content(vec![def], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["exhaust_strike"; 10], &["test_goblin"]);

        let events = combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::CardExhausted { .. })));
        assert_eq!(combat.exhaust.len(), 1);
    }

    #[test]
    fn concentration_card_stays_in_hand() {
        let mut def = defend_card_def();
        def.id = "conc_shield".into();
        def.concentration = true;
        let content = make_content(vec![def, defend_card_def()], vec![test_enemy_def()]);
        let deck: Vec<&str> = std::iter::repeat("conc_shield")
            .take(2)
            .chain(std::iter::repeat("defend").take(8))
            .collect();
        let mut combat = make_combat(&content, &deck, &["test_goblin"]);

        let idx = combat.hand.iter().position(|c| c.def_id == "conc_shield");
        if let Some(idx) = idx {
            combat
                .apply(
                    Action::PlayCard {
                        hand_index: idx,
                        target: None,
                    },
                    &content,
                )
                .unwrap();

            assert!(combat.hand.iter().any(|c| c.def_id == "conc_shield"));
            assert!(combat.concentration_active);
        }
    }

    #[test]
    fn recycle_card_goes_to_draw_pile_bottom() {
        let mut def = defend_card_def();
        def.id = "recycle_block".into();
        def.recycle = true;
        let content = make_content(vec![def, defend_card_def()], vec![test_enemy_def()]);
        let deck: Vec<&str> = std::iter::repeat("recycle_block")
            .take(2)
            .chain(std::iter::repeat("defend").take(8))
            .collect();
        let mut combat = make_combat(&content, &deck, &["test_goblin"]);

        let idx = combat.hand.iter().position(|c| c.def_id == "recycle_block");
        if let Some(idx) = idx {
            let draw_before = combat.draw_pile.len();
            combat
                .apply(
                    Action::PlayCard {
                        hand_index: idx,
                        target: None,
                    },
                    &content,
                )
                .unwrap();

            // Card went to draw pile, not discard.
            assert_eq!(combat.draw_pile.len(), draw_before + 1);
            assert_eq!(combat.draw_pile[0].def_id, "recycle_block");
            assert!(combat.discard.is_empty());
        }
    }

    // ── Damage pipeline ─────────────────────────────────────────────────

    #[test]
    fn empowered_adds_flat_damage() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.player.status_effects.apply(StatusType::Empowered, 3);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.enemies[0].hp, 7); // 10 + 3 = 13 damage
    }

    #[test]
    fn weakened_reduces_damage() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.player.status_effects.apply(StatusType::Weakened, 1);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.enemies[0].hp, 13); // 10 × 0.75 = 7
    }

    #[test]
    fn threatened_amplifies_damage() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.enemies[0]
            .status_effects
            .apply(StatusType::Threatened, 1);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.enemies[0].hp, 5); // 10 × 1.5 = 15
    }

    #[test]
    fn marked_adds_flat_damage() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.enemies[0]
            .status_effects
            .apply(StatusType::Marked, 2);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.enemies[0].hp, 8); // 10 + 2 = 12
    }

    #[test]
    fn calculate_damage_full_pipeline() {
        let mut statuses = StatusMap::new();
        statuses.apply(StatusType::Empowered, 2);
        // base 10 + 2 empowered = 12, threatened ×1.5 = 18, + 3 marked = 21
        assert_eq!(
            CombatState::calculate_damage(10, &statuses, true, 3, false),
            21
        );
    }

    #[test]
    fn calculate_damage_miss_is_zero() {
        let statuses = StatusMap::new();
        assert_eq!(
            CombatState::calculate_damage(0, &statuses, true, 5, false),
            0
        );
    }

    // ── Status effects ──────────────────────────────────────────────────

    #[test]
    fn frightened_enemy_skips_action() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 20], &["test_goblin"]);
        combat.enemies[0]
            .status_effects
            .apply(StatusType::Frightened, 1);
        let hp_before = combat.player.hp;

        let events = combat.apply(Action::EndTurn, &content).unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::EnemySkipped { enemy_index: 0 })));
        assert_eq!(combat.player.hp, hp_before);
    }

    #[test]
    fn enemy_bleeding_dot_damage() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 20], &["test_goblin"]);
        combat.enemies[0]
            .status_effects
            .apply(StatusType::Bleeding, 5);

        combat.apply(Action::EndTurn, &content).unwrap();

        // Bleeding deals 5 dot damage and decays to 4.
        assert!(combat.enemies[0].hp <= 15);
        assert_eq!(
            combat.enemies[0].status_effects.get(StatusType::Bleeding),
            4
        );
    }

    // ── Legal actions ───────────────────────────────────────────────────

    #[test]
    fn legal_actions_include_targeted_and_untargeted() {
        let content = make_content(
            vec![strike_def(), defend_card_def()],
            vec![test_enemy_def()],
        );
        let deck: Vec<&str> = vec!["strike"; 10];
        let mut combat = make_combat(&content, &deck, &["test_goblin"]);

        // Manually ensure the hand has both card types.
        combat.hand.push(CardInstance {
            def_id: "defend".into(),
            upgraded: false,
        });

        let actions = combat.legal_actions(&content);
        assert!(actions.contains(&Action::EndTurn));
        assert!(actions.iter().any(|a| matches!(
            a,
            Action::PlayCard {
                target: Some(_),
                ..
            }
        )));
        assert!(actions
            .iter()
            .any(|a| matches!(a, Action::PlayCard { target: None, .. })));
    }

    #[test]
    fn legal_actions_forced_discard() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["strike"; 10], &["test_goblin"]);
        combat.pending_discards = 2;

        let actions = combat.legal_actions(&content);
        assert!(actions
            .iter()
            .all(|a| matches!(a, Action::DiscardCard { .. })));
        assert!(!actions.contains(&Action::EndTurn));
    }

    // ── Block absorption during enemy turn ──────────────────────────────

    #[test]
    fn player_block_absorbs_enemy_damage() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 20], &["test_goblin"]);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();
        assert_eq!(combat.player.block, 10);

        let hp_before = combat.player.hp;
        combat.apply(Action::EndTurn, &content).unwrap();

        // All attacks hit — enemy deals 6 damage, block absorbs most.
        assert!(combat.player.hp >= hp_before - 6);
    }

    // ── Barbed reflects damage ───────────────────────────────────────

    #[test]
    fn barbed_reflects_damage_to_attacker() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 20], &["test_goblin"]);
        combat.player.status_effects.apply(StatusType::Barbed, 3);
        combat.player.block = 0;

        let enemy_hp_before = combat.enemies[0].hp;
        combat.apply(Action::EndTurn, &content).unwrap();

        // All attacks hit, so Barbed always reflects.
        assert!(combat.enemies[0].hp < enemy_hp_before);
    }

    // ── Multi-enemy combat ──────────────────────────────────────────────

    #[test]
    fn multiple_enemies_act_independently() {
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(
            &content,
            &vec!["defend"; 20],
            &["test_goblin", "test_goblin"],
        );

        let events = combat.apply(Action::EndTurn, &content).unwrap();

        let enemy_actions: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, GameEvent::EnemyAction { .. }))
            .collect();
        assert_eq!(enemy_actions.len(), 2);
    }

    #[test]
    fn dead_enemy_skipped_in_turn() {
        let content = make_content(vec![strike_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(
            &content,
            &vec!["strike"; 20],
            &["test_goblin", "test_goblin"],
        );
        combat.enemies[0].hp = 0; // first enemy already dead

        let events = combat.apply(Action::EndTurn, &content).unwrap();

        let enemy_actions: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, GameEvent::EnemyAction { .. }))
            .collect();
        assert_eq!(enemy_actions.len(), 1); // only second enemy acts
    }

    // ── Energy / heal effects ───────────────────────────────────────────

    #[test]
    fn heal_effect() {
        let mut heal_card = defend_card_def();
        heal_card.id = "heal_potion".into();
        heal_card.effects = vec![Effect::Heal { amount: 15 }];
        let content = make_content(vec![heal_card], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["heal_potion"; 10], &["test_goblin"]);
        combat.player.hp = 30;

        let events = combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();

        assert!(events.iter().any(|e| matches!(e, GameEvent::Healed { .. })));
        assert_eq!(combat.player.hp, 45);
    }

    #[test]
    fn energy_gain_effect() {
        let mut energy_card = defend_card_def();
        energy_card.id = "energy_potion".into();
        energy_card.cost = 0;
        energy_card.effects = vec![Effect::GainEnergy { amount: 2 }];
        let content = make_content(vec![energy_card], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["energy_potion"; 10], &["test_goblin"]);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.player.energy, 5); // 3 starting + 2
    }

    // ── New effect tests ────────────────────────────────────────────────

    #[test]
    fn gain_block_conditional_no_existing_block() {
        let mut card = defend_card_def();
        card.id = "brace".into();
        card.effects = vec![Effect::GainBlockConditional {
            amount: 8,
            bonus_amount: 12,
        }];
        let content = make_content(vec![card], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["brace"; 10], &["test_goblin"]);
        assert_eq!(combat.player.block, 0);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.player.block, 8);
    }

    #[test]
    fn gain_block_conditional_with_existing_block() {
        let mut card = defend_card_def();
        card.id = "brace".into();
        card.effects = vec![Effect::GainBlockConditional {
            amount: 8,
            bonus_amount: 12,
        }];
        let content = make_content(vec![card], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["brace"; 10], &["test_goblin"]);
        combat.player.block = 5;

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.player.block, 17); // 5 + 12
    }

    #[test]
    fn block_floor_effect() {
        let mut card = defend_card_def();
        card.id = "unbreakable".into();
        card.cost = 0;
        card.exhaust = true;
        card.effects = vec![Effect::BlockFloor { amount: 5 }];
        let content = make_content(vec![card, defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["unbreakable"; 10], &["test_goblin"]);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: None,
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.block_floor, 5);
    }

    #[test]
    fn double_status_effect() {
        let mut card = defend_card_def();
        card.id = "expose".into();
        card.cost = 0;
        card.exhaust = true;
        card.effects = vec![Effect::DoubleStatus {
            target: Target::SingleEnemy,
            status: StatusType::Marked,
        }];
        let content = make_content(vec![card], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["expose"; 10], &["test_goblin"]);
        combat.enemies[0]
            .status_effects
            .apply(StatusType::Marked, 3);

        combat
            .apply(
                Action::PlayCard {
                    hand_index: 0,
                    target: Some(0),
                },
                &content,
            )
            .unwrap();

        assert_eq!(combat.enemies[0].status_effects.get(StatusType::Marked), 6);
    }

    // ── Stalemate guard tests ──────────────────────────────────────────

    #[test]
    fn stalemate_ends_in_defeat_after_max_turns() {
        // All-defend deck vs a goblin: player blocks forever, never deals damage.
        let content = make_content(vec![defend_card_def()], vec![test_enemy_def()]);
        let mut combat = make_combat(&content, &vec!["defend"; 20], &["test_goblin"]);

        // Fast-forward to turn MAX_TURNS by ending turns repeatedly.
        for _ in 0..MAX_TURNS + 50 {
            if matches!(combat.phase, CombatPhase::Victory | CombatPhase::Defeat) {
                break;
            }
            combat.apply(Action::EndTurn, &content).unwrap();
        }

        assert!(matches!(combat.phase, CombatPhase::Defeat));
        assert!(combat.turn <= MAX_TURNS + 1);
    }
}
