//! Gauntlet game mode — a 50-fight sequence of combat encounters with
//! progressive difficulty scaling.
//!
//! The [`GauntletRunner`] provides a unified `legal_actions()` / `apply()`
//! interface that wraps both combat decisions and between-fight reward
//! choices. This is the primary entry point for bots, RL agents, and the
//! simulation worker.
//!
//! # Example
//!
//! ```ignore
//! let mut g = GauntletRunner::new("two_handed", 42);
//! loop {
//!     let actions = g.legal_actions();
//!     if actions.is_empty() { break; }
//!     let events = g.apply(actions[0].clone());
//! }
//! println!("Fights won: {}", g.fights_won);
//! ```

pub mod observation;

use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

use decker_content::cards::{
    auto_grants_at_level_filtered, generate_rewards, reward_pool_at_level_filtered,
};
use decker_content::encounters;
use decker_content::starter_decks::fighter_starter_deck;
use decker_engine::card::CardInstance;
use decker_engine::card_ids::CardId;
use decker_engine::class::Class;
use decker_engine::combat::{
    Action, ActionError, CombatPhase, CombatState, EnemyScaling, GameEvent,
};
use decker_engine::content_tables::ContentTables;
use decker_engine::enemy::{scale_enemy_def_custom, EnemyState};
use decker_engine::player::PlayerState;
use decker_engine::rng::GameRng;
use decker_engine::subclass::fighter_subclasses;

// ── Constants ────────────────────────────────────────────────────────────

/// Max HP gained on level-up.
const LEVEL_UP_HP_BONUS: i32 = 5;

/// Number of reward cards offered after each fight.
const REWARD_CARD_COUNT: usize = 3;

/// Maximum fights to win the gauntlet — reaching this count is a true victory.
const MAX_FIGHTS: u32 = 50;

/// Play deck is always exactly this size.
pub const PLAY_DECK_SIZE: usize = 16;

/// Maximum collection size.
const MAX_COLLECTION_SIZE: usize = 40;

/// HP percentage healed at rest stops (0.0–1.0).
const REST_STOP_HEAL_FRACTION: f64 = 0.30;

// ── Perk types ──────────────────────────────────────────────────────────

/// Perks available at perk grant levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerkType {
    /// +1 card drawn per turn.
    DrawBonus,
    /// Choose a card in play deck to make innate.
    InnateChoice,
    /// +1 max energy.
    EnergyBonus,
}

// ── Post-fight event types ──────────────────────────────────────────────

/// What happens after each fight, determined by the event schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostFightEvent {
    /// Standard battle reward: pick or skip.
    BattleReward,
    /// Auto-grant one or more cards from the progression table.
    CardGrant { level: u32, card_ids: Vec<CardId> },
    /// Perk grant at specific levels.
    PerkGrant { level: u32, perk: PerkType },
    /// Rest stop: auto-heal, then optionally rebuild deck.
    RestStop,
}

/// What happens after resolving a collection-overflow trim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverflowNextPhase {
    /// Resume the run by starting the next combat.
    NextCombat,
    /// Offer an optional immediate deck sub-in for the newly granted card.
    DeckSwap {
        new_card_id: CardId,
        new_card_index: usize,
    },
}

/// Fight numbers (1-indexed, fights_won value) that trigger a level-up.
/// Each entry is the fights_won value at which the player reaches the next level.
/// Level 1 is the starting level; fight 4 → level 2, fight 8 → level 3, etc.
const LEVEL_UP_FIGHTS: &[u32] = &[4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44];

/// Fight numbers that trigger a rest stop (heal + optional deck rebuild).
const REST_STOP_FIGHTS: &[u32] = &[7, 15, 23, 31, 39, 47];

/// Level-based rewards. Perks are defined by level, not fight number.
fn level_reward(level: u32, subclass_id: &str, synergy_filter: Option<&str>) -> PostFightEvent {
    match level {
        5  => PostFightEvent::PerkGrant { level, perk: PerkType::DrawBonus },
        8  => PostFightEvent::PerkGrant { level, perk: PerkType::InnateChoice },
        10 => PostFightEvent::PerkGrant { level, perk: PerkType::EnergyBonus },
        _  => {
            let card_ids = auto_grants_at_level_filtered(subclass_id, level, synergy_filter);
            PostFightEvent::CardGrant { level, card_ids }
        }
    }
}

/// Look up the post-fight event for a given fight number (1-indexed, after winning that fight).
///
/// The schedule is: level-ups at fixed fight intervals, rest stops at fixed fights,
/// and everything else is a battle reward. Perks and card grants are determined
/// by the level reached, not the fight number directly.
fn post_fight_event(fights_won: u32, subclass_id: &str, synergy_filter: Option<&str>) -> PostFightEvent {
    // Rest stops take priority (they don't grant a level)
    if REST_STOP_FIGHTS.contains(&fights_won) {
        return PostFightEvent::RestStop;
    }

    // Check if this fight triggers a level-up
    if let Some(pos) = LEVEL_UP_FIGHTS.iter().position(|&f| f == fights_won) {
        let new_level = (pos as u32) + 2; // first entry is level 2
        return level_reward(new_level, subclass_id, synergy_filter);
    }

    PostFightEvent::BattleReward
}

/// Get the player level based on fights won (checking all milestone fights).
pub fn player_level_at(fights_won: u32, subclass_id: &str) -> u32 {
    let mut level = 1u32;
    for f in 1..=fights_won {
        // Synergy filter doesn't affect level progression — only card grants.
        // Pass None so all CardGrant/PerkGrant milestones are counted.
        match post_fight_event(f, subclass_id, None) {
            PostFightEvent::CardGrant { .. } | PostFightEvent::PerkGrant { .. } => {
                level += 1;
            }
            _ => {}
        }
    }
    level
}

// ── Gauntlet phase ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GauntletPhase {
    /// In combat. Use `CombatAction` to interact.
    Combat,
    /// Choosing a reward card or skipping.
    Reward,
    /// Resolve collection overflow by permanently removing one owned card.
    CollectionOverflow { next_phase: OverflowNextPhase },
    /// Choose a deck card to make innate (InnateChoice perk).
    InnateChoice,
    /// Swap a newly acquired card into the play deck (or skip).
    DeckSwap {
        /// The card just acquired (in collection, not yet in play deck).
        new_card_id: CardId,
    },
    /// Ordered 16-slot short-rest rebuild from the full collection.
    DeckRebuild {
        slot_idx: usize,
        partial_deck: Vec<CardInstance>,
    },
    /// Run is over (player died or won).
    GameOver,
}

// ── Gauntlet actions and events ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GauntletAction {
    /// A combat action forwarded to the inner `CombatState`.
    CombatAction(Action),
    /// Pick one of the offered reward cards (index into `reward_options`).
    PickReward(usize),
    /// Skip the reward and proceed to the next fight.
    SkipReward,
    /// Remove one card from the full collection to resolve overflow.
    RemoveCollectionCard(usize),
    /// Choose a card in play deck to make innate (index into `play_deck`).
    ChooseInnate(usize),
    /// Swap newly acquired card into a deck slot (index into `play_deck`).
    SwapIntoDeck(usize),
    /// Keep deck as-is after card acquisition.
    SkipSwap,
    /// Choose the card that should occupy the current rebuild slot.
    ChooseDeckSlotCard(usize),
}

#[derive(Debug, Clone)]
pub enum GauntletEvent {
    /// Events from the inner combat system.
    Combat(Vec<GameEvent>),
    /// Combat ended in victory; transitioning to next phase.
    FightWon {
        fights_won: u32,
        player_hp: i32,
        player_max_hp: i32,
    },
    /// Combat ended in defeat; game over.
    FightLost {
        fights_won: u32,
        player_hp: i32,
        player_max_hp: i32,
    },
    /// A reward card was added to the collection.
    RewardPicked { card_id: CardId },
    /// A card was removed from the full collection.
    CollectionCardRemoved { card_id: CardId },
    /// Reward was skipped.
    RewardSkipped,
    /// Player leveled up (+5 max HP, no heal).
    LevelUp { level: u32, new_max_hp: i32 },
    /// A new combat is starting.
    CombatStarted {
        fight_number: u32,
        enemy_count: usize,
    },
    /// Player won the gauntlet by reaching MAX_FIGHTS victories.
    RunWon { fights_won: u32 },
    /// A fixed card was granted (class or subclass progression).
    CardGranted { card_id: CardId },
    /// Player healed at a rest stop.
    RestHealed { amount: i32 },
    /// A perk was granted.
    PerkGranted { perk: PerkType },
    /// A card was made innate via InnateChoice perk.
    CardMadeInnate { card_id: CardId },
    /// A card was swapped into the play deck.
    DeckSwapped {
        added_to_deck: CardId,
        removed_from_deck: CardId,
    },
    /// Deck swap was skipped (card stays in collection only).
    SwapSkipped,
    /// A card was assigned to one short-rest rebuild slot.
    DeckRewriteSlotChosen { slot_idx: usize, card_id: CardId },
    /// Rebuild finished.
    RebuildFinished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GauntletError {
    /// Invalid action for the current phase.
    WrongPhase,
    /// Combat action error (forwarded from CombatState).
    CombatError(ActionError),
    /// Invalid reward index.
    InvalidRewardIndex,
    /// Invalid deck index for card removal.
    InvalidDeckIndex,
    /// Invalid collection index.
    InvalidCollectionIndex,
}

// ── Scaling ──────────────────────────────────────────────────────────────

/// Enemy HP multiplier scaling with fights won.
#[cfg(test)]
fn gauntlet_hp_mult(_fights_won: u32) -> f64 {
    1.0
}

/// Enemy damage multiplier scaling with fights won.
#[cfg(test)]
fn gauntlet_dmg_mult(_fights_won: u32) -> f64 {
    1.0
}

/// Effective act for proficiency and encounter pool purposes.
fn effective_act(fights_won: u32) -> u32 {
    1 + (fights_won / 6).min(2)
}

// ── GauntletRunner ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GauntletRunner {
    /// The player's persistent state (HP carries over between fights).
    pub player: PlayerState,
    /// The player's collection of all owned cards (max 40).
    pub collection: Vec<CardInstance>,
    /// The player's play deck — exactly 16 cards, subset of collection.
    pub play_deck: Vec<CardInstance>,
    /// Total fights won so far.
    pub fights_won: u32,
    /// Current phase of the gauntlet.
    pub phase: GauntletPhase,
    /// Master RNG for encounter generation, rewards, etc.
    pub rng: GameRng,
    /// The sub-class chosen for this run.
    pub subclass_id: String,
    /// Immutable content tables — rebuilt from code on load, not stored in saves.
    #[serde(skip, default = "ContentTables::empty")]
    pub content: ContentTables,
    /// Active combat state (only present during Combat phase).
    pub combat: Option<CombatState>,
    /// Reward cards offered after a fight (only present during Reward phase).
    pub reward_options: Vec<CardId>,
    /// Current player level (1-indexed, starts at 1).
    pub level: u32,
    /// Bonus draw per turn from DrawBonus perk.
    pub bonus_draw: i32,
    /// Bonus energy from EnergyBonus perk.
    pub bonus_energy: i32,
    /// Cards made innate by InnateChoice perk.
    pub innate_overrides: HashSet<CardId>,
    /// Optional synergy group filter (e.g. "marked", "tempo").
    /// When set, the reward pool and auto-grants only include cards from this
    /// synergy group (plus neutral/shared cards with `synergy: None`).
    pub synergy_filter: Option<String>,
    /// The player's race (determines starter race card).
    pub race: String,
    /// The player's background (determines starter background card).
    pub background: String,
    /// The player's class.
    pub class: Class,
}

impl GauntletRunner {
    /// Create a new gauntlet run for the given Fighter sub-class and RNG seed.
    /// Defaults to human race and soldier background.
    pub fn new(subclass_id: &str, seed: u64) -> Self {
        Self::new_full(subclass_id, seed, None, "human", "soldier")
    }

    /// Create a new gauntlet run with an optional synergy group filter.
    /// Defaults to human race and soldier background.
    pub fn new_filtered(subclass_id: &str, seed: u64, synergy_filter: Option<&str>) -> Self {
        Self::new_full(subclass_id, seed, synergy_filter, "human", "soldier")
    }

    /// Create a new gauntlet run with all options.
    pub fn new_full(subclass_id: &str, seed: u64, synergy_filter: Option<&str>, race: &str, background: &str) -> Self {
        let content = decker_content::load_content();
        let rng = GameRng::new(seed);

        // Determine class from subclass ID.
        let all_subclasses = fighter_subclasses();
        let subclass_def = all_subclasses
            .iter()
            .find(|s| s.id == subclass_id)
            .unwrap_or_else(|| panic!("unknown sub-class: {}", subclass_id));
        let class = subclass_def.class;

        // Build player state.
        let player = PlayerState::new(
            class.base_hp(),
            3, // starting energy
        );

        // Build starter deck (also serves as initial collection).
        let starter = fighter_starter_deck(subclass_id, race, background);

        // Create first combat.
        let mut runner = Self {
            player,
            collection: starter.clone(),
            play_deck: starter,
            fights_won: 0,
            phase: GauntletPhase::Combat,
            rng,
            subclass_id: subclass_id.to_string(),
            content,
            combat: None,
            reward_options: Vec::new(),
            level: 1,
            bonus_draw: 0,
            bonus_energy: 0,
            innate_overrides: HashSet::new(),
            synergy_filter: synergy_filter.map(|s| s.to_string()),
            race: race.to_string(),
            background: background.to_string(),
            class,
        };
        runner.start_combat();
        runner
    }

    /// Reconstruct a runner from a save file JSON string.
    /// Content tables are rebuilt from code (they're skipped in saves).
    pub fn from_save(json: &str) -> Result<Self, String> {
        let mut runner: GauntletRunner = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize save: {}", e))?;
        runner.content = decker_content::load_content();
        Ok(runner)
    }

    // ── Public API ───────────────────────────────────────────────────────

    /// Returns all legal actions for the current phase.
    pub fn legal_actions(&self) -> Vec<GauntletAction> {
        match &self.phase {
            GauntletPhase::Combat => {
                if let Some(combat) = &self.combat {
                    combat
                        .legal_actions(&self.content)
                        .into_iter()
                        .map(GauntletAction::CombatAction)
                        .collect()
                } else {
                    vec![]
                }
            }
            GauntletPhase::Reward => {
                let mut actions = Vec::new();
                for i in 0..self.reward_options.len() {
                    actions.push(GauntletAction::PickReward(i));
                }
                actions.push(GauntletAction::SkipReward);
                actions
            }
            GauntletPhase::CollectionOverflow { .. } => (0..self.overflow_removal_indices().len())
                .map(GauntletAction::RemoveCollectionCard)
                .collect(),
            GauntletPhase::InnateChoice => {
                let mut actions = Vec::new();
                for i in 0..self.play_deck.len() {
                    actions.push(GauntletAction::ChooseInnate(i));
                }
                actions
            }
            GauntletPhase::DeckSwap { .. } => {
                let mut actions = Vec::new();
                let new_card_id = match &self.phase {
                    GauntletPhase::DeckSwap { new_card_id } => new_card_id.as_str(),
                    _ => unreachable!(),
                };
                for i in 0..self.play_deck.len() {
                    if self.can_swap_new_card_into_slot(new_card_id, i) {
                        actions.push(GauntletAction::SwapIntoDeck(i));
                    }
                }
                actions.push(GauntletAction::SkipSwap);
                actions
            }
            GauntletPhase::DeckRebuild { partial_deck, .. } => self
                .rebuild_card_choices(partial_deck)
                .iter()
                .enumerate()
                .map(|(i, _)| GauntletAction::ChooseDeckSlotCard(i))
                .collect(),
            GauntletPhase::GameOver => vec![],
        }
    }

    /// Apply a gauntlet action and return the resulting events.
    pub fn apply(&mut self, action: GauntletAction) -> Result<Vec<GauntletEvent>, GauntletError> {
        match (&self.phase.clone(), action) {
            (GauntletPhase::Combat, GauntletAction::CombatAction(combat_action)) => {
                self.apply_combat_action(combat_action)
            }
            (GauntletPhase::Reward, GauntletAction::PickReward(idx)) => self.apply_pick_reward(idx),
            (GauntletPhase::Reward, GauntletAction::SkipReward) => self.apply_skip_reward(),
            (
                GauntletPhase::CollectionOverflow { .. },
                GauntletAction::RemoveCollectionCard(idx),
            ) => self.apply_remove_collection_card(idx),
            (GauntletPhase::InnateChoice, GauntletAction::ChooseInnate(idx)) => {
                self.apply_choose_innate(idx)
            }
            (GauntletPhase::DeckSwap { .. }, GauntletAction::SwapIntoDeck(idx)) => {
                self.apply_swap_into_deck(idx)
            }
            (GauntletPhase::DeckSwap { .. }, GauntletAction::SkipSwap) => self.apply_skip_swap(),
            (GauntletPhase::DeckRebuild { .. }, GauntletAction::ChooseDeckSlotCard(idx)) => {
                self.apply_choose_deck_slot_card(idx)
            }
            _ => Err(GauntletError::WrongPhase),
        }
    }

    /// Returns true if the run is over.
    pub fn is_game_over(&self) -> bool {
        self.phase == GauntletPhase::GameOver
    }

    fn collection_counts(cards: &[CardInstance]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.def_id.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn card_is_legendary(&self, card_id: &str) -> bool {
        self.content
            .card_defs
            .get(card_id)
            .map(|def| def.rarity == decker_engine::card::Rarity::Legendary)
            .unwrap_or(false)
    }

    fn card_is_inherent(&self, card_id: &str) -> bool {
        self.content
            .card_defs
            .get(card_id)
            .map(|def| def.inherent)
            .unwrap_or(false)
    }

    fn legendary_count(&self, cards: &[CardInstance]) -> usize {
        cards
            .iter()
            .filter(|card| self.card_is_legendary(&card.def_id))
            .count()
    }

    fn rebuild_card_choices(&self, partial_deck: &[CardInstance]) -> Vec<CardId> {
        let coll_counts = Self::collection_counts(&self.collection);
        let partial_counts = Self::collection_counts(partial_deck);

        // Inherent cards must be placed first. If any inherent cards are in
        // the collection but not yet in the partial deck, only offer those.
        let mut unplaced_inherent = Vec::new();
        let mut seen_inherent = HashSet::new();
        for card in &self.collection {
            if !self.card_is_inherent(&card.def_id) {
                continue;
            }
            if !seen_inherent.insert(card.def_id.clone()) {
                continue;
            }
            let available = coll_counts.get(&card.def_id).copied().unwrap_or(0);
            let already_used = partial_counts.get(&card.def_id).copied().unwrap_or(0);
            if already_used < available {
                unplaced_inherent.push(card.def_id.clone());
            }
        }
        if !unplaced_inherent.is_empty() {
            return unplaced_inherent;
        }

        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for card in &self.collection {
            if !seen.insert(card.def_id.clone()) {
                continue;
            }
            let available = coll_counts.get(&card.def_id).copied().unwrap_or(0);
            let already_used = partial_counts.get(&card.def_id).copied().unwrap_or(0);
            if already_used >= available {
                continue;
            }

            let mut candidate = partial_deck.to_vec();
            candidate.push(CardInstance {
                def_id: card.def_id.clone(),
                upgraded: false,
            });
            if self.partial_rebuild_is_feasible(&candidate) {
                result.push(card.def_id.clone());
            }
        }

        result
    }

    fn partial_rebuild_is_feasible(&self, partial_deck: &[CardInstance]) -> bool {
        if partial_deck.len() > PLAY_DECK_SIZE {
            return false;
        }
        let legendary_count = self.legendary_count(partial_deck);
        if legendary_count > 1 {
            return false;
        }

        let coll_counts = Self::collection_counts(&self.collection);
        let partial_counts = Self::collection_counts(partial_deck);
        for (card_id, &count) in &partial_counts {
            if count > coll_counts.get(card_id).copied().unwrap_or(0) {
                return false;
            }
        }

        let remaining_slots = PLAY_DECK_SIZE - partial_deck.len();
        let mut remaining_total = 0usize;
        let mut remaining_nonlegendary = 0usize;
        for (card_id, &owned) in &coll_counts {
            let used = partial_counts.get(card_id).copied().unwrap_or(0);
            let remaining = owned.saturating_sub(used);
            remaining_total += remaining;
            if !self.card_is_legendary(card_id) {
                remaining_nonlegendary += remaining;
            }
        }

        if remaining_total < remaining_slots {
            return false;
        }
        if legendary_count >= 1 && remaining_nonlegendary < remaining_slots {
            return false;
        }

        true
    }

    fn overflow_removal_indices(&self) -> Vec<usize> {
        let deck_counts = Self::collection_counts(&self.play_deck);
        let collection_counts = Self::collection_counts(&self.collection);
        let mut removable = Vec::new();

        for (idx, card) in self.collection.iter().enumerate() {
            // Cannot remove inherent cards (race or background).
            if self.card_is_inherent(&card.def_id) {
                continue;
            }
            let mut remaining = collection_counts.clone();
            if let Some(count) = remaining.get_mut(&card.def_id) {
                *count = count.saturating_sub(1);
            }
            let keeps_current_deck_valid = deck_counts
                .iter()
                .all(|(card_id, &needed)| remaining.get(card_id).copied().unwrap_or(0) >= needed);
            if keeps_current_deck_valid {
                removable.push(idx);
            }
        }

        removable
    }

    fn can_swap_new_card_into_slot(&self, new_card_id: &str, deck_idx: usize) -> bool {
        if deck_idx >= self.play_deck.len() {
            return false;
        }
        // Cannot swap out an inherent card (race or background).
        if self.card_is_inherent(&self.play_deck[deck_idx].def_id) {
            return false;
        }
        if !self.card_is_legendary(new_card_id) {
            return true;
        }
        let current_legendary_positions: Vec<usize> = self
            .play_deck
            .iter()
            .enumerate()
            .filter_map(|(i, card)| self.card_is_legendary(&card.def_id).then_some(i))
            .collect();
        current_legendary_positions.is_empty()
            || (current_legendary_positions.len() == 1
                && current_legendary_positions[0] == deck_idx)
    }

    // ── Combat phase ─────────────────────────────────────────────────────

    fn apply_combat_action(&mut self, action: Action) -> Result<Vec<GauntletEvent>, GauntletError> {
        let combat = self.combat.as_mut().ok_or(GauntletError::WrongPhase)?;
        let events = combat
            .apply(action, &self.content)
            .map_err(GauntletError::CombatError)?;

        let mut gauntlet_events = vec![GauntletEvent::Combat(events)];

        // Check if combat ended.
        match combat.phase {
            CombatPhase::Victory => {
                self.finish_combat_victory(&mut gauntlet_events);
            }
            CombatPhase::Defeat => {
                let player_hp = combat.player.hp;
                let player_max_hp = combat.player.max_hp;
                self.phase = GauntletPhase::GameOver;
                self.combat = None;
                gauntlet_events.push(GauntletEvent::FightLost {
                    fights_won: self.fights_won,
                    player_hp,
                    player_max_hp,
                });
            }
            _ => {}
        }

        Ok(gauntlet_events)
    }

    fn finish_combat_victory(&mut self, events: &mut Vec<GauntletEvent>) {
        // Carry over HP from combat.
        let mut player_hp = self.player.hp;
        let mut player_max_hp = self.player.max_hp;
        if let Some(combat) = &self.combat {
            player_hp = combat.player.hp;
            player_max_hp = combat.player.max_hp;
            self.player.hp = combat.player.hp;
        }
        self.combat = None;
        self.fights_won += 1;

        events.push(GauntletEvent::FightWon {
            fights_won: self.fights_won,
            player_hp,
            player_max_hp,
        });

        // Win condition: player completed the gauntlet.
        if self.fights_won >= MAX_FIGHTS {
            events.push(GauntletEvent::RunWon {
                fights_won: self.fights_won,
            });
            self.phase = GauntletPhase::GameOver;
            return;
        }

        // Look up what happens after this fight.
        let event = post_fight_event(self.fights_won, &self.subclass_id, self.synergy_filter.as_deref());
        match event {
            PostFightEvent::BattleReward => {
                self.generate_rewards();
                self.phase = GauntletPhase::Reward;
            }
            PostFightEvent::CardGrant { level, card_ids } => {
                self.apply_level_up(level, events);
                if card_ids.is_empty() {
                    // No auto-grants at this level (e.g. dueling level 3 has
                    // pool-only cards). Go straight to the next fight.
                    self.transition_to_next_combat(events);
                } else {
                    for card_id in &card_ids {
                        self.grant_card(card_id.clone(), events);
                    }
                }
            }
            PostFightEvent::PerkGrant { level, perk } => {
                self.apply_level_up(level, events);
                self.apply_perk(perk, events);
            }
            PostFightEvent::RestStop => {
                self.auto_rest_stop(events);
            }
        }
    }

    /// Apply a level-up: +5 max HP, no heal.
    fn apply_level_up(&mut self, level: u32, events: &mut Vec<GauntletEvent>) {
        self.level = level;
        self.player.max_hp += LEVEL_UP_HP_BONUS;
        events.push(GauntletEvent::LevelUp {
            level,
            new_max_hp: self.player.max_hp,
        });
    }

    /// Grant a fixed card: add to collection, resolve overflow if needed, then offer DeckSwap.
    fn grant_card(&mut self, card_id: CardId, events: &mut Vec<GauntletEvent>) {
        let card = CardInstance {
            def_id: card_id.clone(),
            upgraded: false,
        };
        self.collection.push(card);
        events.push(GauntletEvent::CardGranted {
            card_id: card_id.clone(),
        });
        let new_card_index = self.collection.len() - 1;
        if self.collection.len() > MAX_COLLECTION_SIZE {
            self.phase = GauntletPhase::CollectionOverflow {
                next_phase: OverflowNextPhase::DeckSwap {
                    new_card_id: card_id,
                    new_card_index,
                },
            };
        } else {
            self.phase = GauntletPhase::DeckSwap {
                new_card_id: card_id,
            };
        }
    }

    /// Apply a perk.
    fn apply_perk(&mut self, perk: PerkType, events: &mut Vec<GauntletEvent>) {
        match perk {
            PerkType::DrawBonus => {
                self.bonus_draw += 1;
            }
            PerkType::EnergyBonus => {
                self.player.max_energy += 1;
            }
            PerkType::InnateChoice => {
                events.push(GauntletEvent::PerkGranted { perk });
                self.phase = GauntletPhase::InnateChoice;
                return;
            }
        }
        events.push(GauntletEvent::PerkGranted { perk });
        self.transition_to_next_combat(events);
    }

    // ── InnateChoice phase ───────────────────────────────────────────────

    fn apply_choose_innate(&mut self, idx: usize) -> Result<Vec<GauntletEvent>, GauntletError> {
        if idx >= self.play_deck.len() {
            return Err(GauntletError::InvalidDeckIndex);
        }
        let card_id = self.play_deck[idx].def_id.clone();
        self.innate_overrides.insert(card_id.clone());
        let mut events = vec![GauntletEvent::CardMadeInnate { card_id }];
        self.transition_to_next_combat(&mut events);
        Ok(events)
    }

    // ── DeckSwap phase ───────────────────────────────────────────────────

    fn apply_swap_into_deck(
        &mut self,
        deck_idx: usize,
    ) -> Result<Vec<GauntletEvent>, GauntletError> {
        if deck_idx >= self.play_deck.len() {
            return Err(GauntletError::InvalidDeckIndex);
        }

        let new_card_id = match &self.phase {
            GauntletPhase::DeckSwap { new_card_id } => new_card_id.clone(),
            _ => return Err(GauntletError::WrongPhase),
        };
        if !self.can_swap_new_card_into_slot(&new_card_id, deck_idx) {
            return Err(GauntletError::InvalidDeckIndex);
        }

        let removed = self.play_deck[deck_idx].clone();
        // Find the new card in collection and put it in the deck slot
        let new_card = CardInstance {
            def_id: new_card_id.clone(),
            upgraded: false,
        };
        self.play_deck[deck_idx] = new_card;

        let mut events = vec![GauntletEvent::DeckSwapped {
            added_to_deck: new_card_id,
            removed_from_deck: removed.def_id,
        }];
        self.transition_to_next_combat(&mut events);
        Ok(events)
    }

    fn apply_skip_swap(&mut self) -> Result<Vec<GauntletEvent>, GauntletError> {
        let mut events = vec![GauntletEvent::SwapSkipped];
        self.transition_to_next_combat(&mut events);
        Ok(events)
    }

    // ── DeckRebuild phase ────────────────────────────────────────────────

    fn apply_choose_deck_slot_card(
        &mut self,
        choice_idx: usize,
    ) -> Result<Vec<GauntletEvent>, GauntletError> {
        let (slot_idx, mut partial_deck) = match &self.phase {
            GauntletPhase::DeckRebuild {
                slot_idx,
                partial_deck,
            } => (*slot_idx, partial_deck.clone()),
            _ => return Err(GauntletError::WrongPhase),
        };

        let choices = self.rebuild_card_choices(&partial_deck);
        if choice_idx >= choices.len() {
            return Err(GauntletError::InvalidCollectionIndex);
        }
        let card_id = choices[choice_idx].clone();
        partial_deck.push(CardInstance {
            def_id: card_id.clone(),
            upgraded: false,
        });

        let mut events = vec![GauntletEvent::DeckRewriteSlotChosen { slot_idx, card_id }];
        if partial_deck.len() >= PLAY_DECK_SIZE {
            self.play_deck = partial_deck;
            events.push(GauntletEvent::RebuildFinished);
            self.transition_to_next_combat(&mut events);
        } else {
            self.phase = GauntletPhase::DeckRebuild {
                slot_idx: slot_idx + 1,
                partial_deck,
            };
        }

        Ok(events)
    }

    // ── Reward phase ─────────────────────────────────────────────────────

    fn apply_pick_reward(&mut self, idx: usize) -> Result<Vec<GauntletEvent>, GauntletError> {
        if idx >= self.reward_options.len() {
            return Err(GauntletError::InvalidRewardIndex);
        }
        let card_id = self.reward_options[idx].clone();

        // Add to collection
        self.collection.push(CardInstance {
            def_id: card_id.clone(),
            upgraded: false,
        });
        self.reward_options.clear();

        let mut events = vec![GauntletEvent::RewardPicked {
            card_id: card_id.clone(),
        }];
        if self.collection.len() > MAX_COLLECTION_SIZE {
            self.phase = GauntletPhase::CollectionOverflow {
                next_phase: OverflowNextPhase::NextCombat,
            };
        } else {
            self.transition_to_next_combat(&mut events);
        }
        Ok(events)
    }

    fn apply_skip_reward(&mut self) -> Result<Vec<GauntletEvent>, GauntletError> {
        self.reward_options.clear();
        let mut events = vec![GauntletEvent::RewardSkipped];
        self.transition_to_next_combat(&mut events);
        Ok(events)
    }

    fn apply_remove_collection_card(
        &mut self,
        choice_idx: usize,
    ) -> Result<Vec<GauntletEvent>, GauntletError> {
        let next_phase = match &self.phase {
            GauntletPhase::CollectionOverflow { next_phase } => next_phase.clone(),
            _ => return Err(GauntletError::WrongPhase),
        };
        let removable = self.overflow_removal_indices();
        if choice_idx >= removable.len() {
            return Err(GauntletError::InvalidCollectionIndex);
        }
        let idx = removable[choice_idx];
        let removed = self.collection.remove(idx);
        let mut events = vec![GauntletEvent::CollectionCardRemoved {
            card_id: removed.def_id.clone(),
        }];

        match next_phase {
            OverflowNextPhase::NextCombat => {
                self.transition_to_next_combat(&mut events);
            }
            OverflowNextPhase::DeckSwap {
                new_card_id,
                new_card_index,
            } => {
                if idx == new_card_index {
                    self.transition_to_next_combat(&mut events);
                } else {
                    self.phase = GauntletPhase::DeckSwap { new_card_id };
                }
            }
        }

        Ok(events)
    }

    // ── Rest stop (automatic) ──────────────────────────────────────────

    /// Automatically heal at rest stop, then enter the ordered deck rewrite
    /// when the collection offers meaningful flexibility.
    fn auto_rest_stop(&mut self, events: &mut Vec<GauntletEvent>) {
        let heal_amount = (self.player.max_hp as f64 * REST_STOP_HEAL_FRACTION).round() as i32;
        let hp_before = self.player.hp;
        self.player.heal(heal_amount);
        let actual_heal = self.player.hp - hp_before;
        events.push(GauntletEvent::RestHealed {
            amount: actual_heal,
        });

        if self.collection.len() > PLAY_DECK_SIZE {
            self.phase = GauntletPhase::DeckRebuild {
                slot_idx: 0,
                partial_deck: Vec::new(),
            };
        } else {
            self.transition_to_next_combat(events);
        }
    }

    // ── Transitions ──────────────────────────────────────────────────────

    fn transition_to_next_combat(&mut self, events: &mut Vec<GauntletEvent>) {
        self.start_combat();
        if let Some(combat) = &self.combat {
            events.push(GauntletEvent::CombatStarted {
                fight_number: self.fights_won + 1,
                enemy_count: combat.enemies.len(),
            });
        }
    }

    // ── Combat setup ─────────────────────────────────────────────────────

    fn start_combat(&mut self) {
        let act = effective_act(self.fights_won);
        let player_level = player_level_at(self.fights_won, &self.subclass_id);

        // Keep authored encounters unscaled while the curated level bands are
        // being balanced. Higher player levels clamp onto the level-3 table.
        let hp_mult = 1.0;
        let dmg_mult = 1.0;

        let enemy_ids =
            encounters::encounter_for_level(player_level, self.fights_won as usize, act, &mut self.rng);

        // Resolve and scale enemy definitions.
        let enemies: Vec<EnemyState> = enemy_ids
            .iter()
            .filter_map(|id| {
                self.content.enemy_defs.get(id).map(|def| {
                    let scaled = scale_enemy_def_custom(def, hp_mult, dmg_mult);
                    EnemyState::from_def(&scaled)
                })
            })
            .collect();

        // Reset player block and status for new combat.
        let mut combat_player = self.player.clone();
        combat_player.block = 0;
        combat_player.energy = combat_player.max_energy;
        combat_player.status_effects = Default::default();

        let combat_seed = self.rng.next_u64();
        let mut combat = CombatState::new_with_act(
            combat_player,
            self.play_deck.clone(),
            enemies,
            combat_seed,
            0, // no bonus draw turn 1
            0, // no bonus energy turn 1
            Some(self.class),
            act,
            EnemyScaling::Custom { hp_mult, dmg_mult },
            &self.content,
        );

        // Apply perk bonuses
        combat.draw_per_turn = (5 + self.bonus_draw) as usize;
        combat.innate_overrides = self.innate_overrides.clone();

        self.combat = Some(combat);
        self.phase = GauntletPhase::Combat;
    }

    fn generate_rewards(&mut self) {
        let filter = self.synergy_filter.as_deref();
        let pool = reward_pool_at_level_filtered(&self.subclass_id, self.level, filter);
        self.reward_options = generate_rewards(&pool, REWARD_CARD_COUNT, &mut self.rng);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_runner_starts_in_combat() {
        let g = GauntletRunner::new("two_handed", 42);
        assert_eq!(g.phase, GauntletPhase::Combat);
        assert!(g.combat.is_some());
        assert_eq!(g.fights_won, 0);
        assert_eq!(g.play_deck.len(), PLAY_DECK_SIZE);
        assert_eq!(g.collection.len(), PLAY_DECK_SIZE);
        assert_eq!(g.level, 1);
    }

    #[test]
    fn legal_actions_in_combat() {
        let g = GauntletRunner::new("defense", 42);
        let actions = g.legal_actions();
        assert!(!actions.is_empty());
        assert!(actions
            .iter()
            .all(|a| matches!(a, GauntletAction::CombatAction(_))));
    }

    #[test]
    fn game_over_has_no_actions() {
        let mut g = GauntletRunner::new("dueling", 42);
        g.phase = GauntletPhase::GameOver;
        g.combat = None;
        assert!(g.legal_actions().is_empty());
        assert!(g.is_game_over());
    }

    #[test]
    fn reward_phase_has_pick_skip_actions() {
        let mut g = GauntletRunner::new("two_handed", 42);
        g.phase = GauntletPhase::Reward;
        g.reward_options = vec!["strike".into(), "defend".into(), "heavy_strike".into()];
        let actions = g.legal_actions();

        assert!(actions.contains(&GauntletAction::PickReward(0)));
        assert!(actions.contains(&GauntletAction::PickReward(1)));
        assert!(actions.contains(&GauntletAction::PickReward(2)));
        assert!(actions.contains(&GauntletAction::SkipReward));
    }

    #[test]
    fn pick_reward_starts_next_combat() {
        let mut g = GauntletRunner::new("two_handed", 42);
        g.phase = GauntletPhase::Reward;
        g.reward_options = vec!["strike".into(), "defend".into(), "heavy_strike".into()];
        let initial_coll = g.collection.len();

        let events = g.apply(GauntletAction::PickReward(1)).unwrap();
        assert_eq!(g.collection.len(), initial_coll + 1);
        assert!(events
            .iter()
            .any(|e| matches!(e, GauntletEvent::RewardPicked { card_id } if card_id == "defend")));
        assert!(events
            .iter()
            .any(|e| matches!(e, GauntletEvent::CombatStarted { .. })));
        assert_eq!(g.phase, GauntletPhase::Combat);
    }

    #[test]
    fn skip_reward_starts_next_combat() {
        let mut g = GauntletRunner::new("defense", 42);
        g.phase = GauntletPhase::Reward;
        g.reward_options = vec!["strike".into()];
        let initial_coll = g.collection.len();

        let events = g.apply(GauntletAction::SkipReward).unwrap();
        assert_eq!(g.collection.len(), initial_coll); // no change
        assert!(events
            .iter()
            .any(|e| matches!(e, GauntletEvent::RewardSkipped)));
        assert_eq!(g.phase, GauntletPhase::Combat);
    }

    #[test]
    fn wrong_phase_rejected() {
        let mut g = GauntletRunner::new("two_handed", 42);
        // In combat phase, reward actions should fail
        let result = g.apply(GauntletAction::SkipReward);
        assert!(matches!(result, Err(GauntletError::WrongPhase)));
    }

    #[test]
    fn no_per_fight_scaling() {
        assert_eq!(gauntlet_hp_mult(0), 1.0);
        assert_eq!(gauntlet_hp_mult(10), 1.0);
        assert_eq!(gauntlet_dmg_mult(0), 1.0);
        assert_eq!(gauntlet_dmg_mult(10), 1.0);
    }

    #[test]
    fn gauntlet_uses_level_aware_pool_without_enemy_scaling() {
        let mut g = GauntletRunner::new("dueling", 42);
        g.fights_won = 8;
        g.level = 3;
        g.start_combat();

        let combat = g.combat.as_ref().expect("combat should be present");
        // Level 3 encounters are curated — collect all valid enemy IDs for this level.
        let all_defs = decker_content::enemies::all_enemy_defs();
        let allowed: HashSet<String> = all_defs.keys().map(|id| id.to_string()).collect();

        for enemy in &combat.enemies {
            assert!(allowed.contains(&enemy.def_id));
            let base = g
                .content
                .enemy_defs
                .get(&enemy.def_id)
                .expect("enemy must exist in content");
            assert_eq!(enemy.max_hp, base.max_hp);
        }
    }

    #[test]
    fn effective_act_caps_at_3() {
        assert_eq!(effective_act(0), 1);
        assert_eq!(effective_act(5), 1);
        assert_eq!(effective_act(6), 2);
        assert_eq!(effective_act(12), 3);
        assert_eq!(effective_act(100), 3);
    }

    #[test]
    fn all_subclasses_can_start() {
        for sc in &["two_handed", "defense", "dueling"] {
            let g = GauntletRunner::new(sc, 42);
            assert_eq!(g.phase, GauntletPhase::Combat);
            assert!(g.combat.is_some());
        }
    }

    #[test]
    fn post_fight_event_schedule() {
        // Spot-check the event schedule
        assert!(matches!(
            post_fight_event(4, "defense", None),
            PostFightEvent::CardGrant { level: 2, .. }
        ));
        assert!(matches!(
            post_fight_event(7, "defense", None),
            PostFightEvent::RestStop
        ));
        assert!(matches!(
            post_fight_event(8, "defense", None),
            PostFightEvent::CardGrant { level: 3, .. }
        ));
        assert!(matches!(
            post_fight_event(16, "defense", None),
            PostFightEvent::PerkGrant {
                level: 5,
                perk: PerkType::DrawBonus
            }
        ));
        assert!(matches!(
            post_fight_event(1, "defense", None),
            PostFightEvent::BattleReward
        ));
    }

    #[test]
    fn player_level_tracking() {
        // After fight 4, player should be level 2
        assert_eq!(player_level_at(4, "defense"), 2);
        // After fight 44, player should be level 12
        assert_eq!(player_level_at(44, "defense"), 12);
        // At fight 0, level 1
        assert_eq!(player_level_at(0, "defense"), 1);
    }

    #[test]
    fn play_deck_stays_at_16() {
        let g = GauntletRunner::new("two_handed", 42);
        assert_eq!(g.play_deck.len(), PLAY_DECK_SIZE);
    }

    #[test]
    fn deck_swap_swaps_card() {
        let mut g = GauntletRunner::new("two_handed", 42);
        let new_card = "energy_surge".to_string();
        g.collection.push(CardInstance {
            def_id: new_card.clone(),
            upgraded: false,
        });
        g.phase = GauntletPhase::DeckSwap {
            new_card_id: new_card.clone(),
        };

        let old_card = g.play_deck[0].def_id.clone();
        let events = g.apply(GauntletAction::SwapIntoDeck(0)).unwrap();
        assert_eq!(g.play_deck[0].def_id, new_card);
        assert_eq!(g.play_deck.len(), PLAY_DECK_SIZE);
        assert!(events.iter().any(
            |e| matches!(e, GauntletEvent::DeckSwapped { added_to_deck, removed_from_deck }
            if added_to_deck == &new_card && removed_from_deck == &old_card)
        ));
        assert_eq!(g.phase, GauntletPhase::Combat);
    }

    #[test]
    fn reward_pick_over_cap_enters_collection_overflow() {
        let mut g = GauntletRunner::new("two_handed", 42);
        g.phase = GauntletPhase::Reward;
        g.reward_options = vec!["defend".into()];
        while g.collection.len() < MAX_COLLECTION_SIZE {
            g.collection.push(CardInstance {
                def_id: "strike".into(),
                upgraded: false,
            });
        }

        let events = g.apply(GauntletAction::PickReward(0)).unwrap();
        assert!(events
            .iter()
            .any(|e| matches!(e, GauntletEvent::RewardPicked { card_id } if card_id == "defend")));
        assert!(matches!(
            g.phase,
            GauntletPhase::CollectionOverflow {
                next_phase: OverflowNextPhase::NextCombat
            }
        ));
    }

    #[test]
    fn rest_stop_enters_ordered_rebuild() {
        let mut g = GauntletRunner::new("two_handed", 42);
        g.collection.push(CardInstance {
            def_id: "power_strike".into(),
            upgraded: false,
        });

        let mut events = Vec::new();
        g.auto_rest_stop(&mut events);
        assert!(events
            .iter()
            .any(|e| matches!(e, GauntletEvent::RestHealed { .. })));
        assert!(matches!(
            g.phase,
            GauntletPhase::DeckRebuild {
                slot_idx: 0,
                partial_deck
            } if partial_deck.is_empty()
        ));
    }

    #[test]
    fn full_run_terminates() {
        let mut g = GauntletRunner::new("two_handed", 99);
        let mut steps = 0;
        let max_steps = 50_000;

        while !g.is_game_over() && steps < max_steps {
            let actions = g.legal_actions();
            if actions.is_empty() {
                break;
            }
            // Always pick first action (simple bot)
            let _ = g.apply(actions[0].clone());
            steps += 1;
        }

        assert!(steps > 0, "should have taken at least one action");
    }
}
