//! Feature encoding for RL agents.
//!
//! Produces normalized, structured feature vectors from the gauntlet state.
//! All features are scaled to roughly [0, 1] or [-1, 1] for stable training.
//!
//! # Observation structure
//!
//! The observation is split into four groups, each padded to fixed maximums
//! so they can be batched across environments without ragged tensors:
//!
//! | Group          | Dim per item | Max items      | Description                       |
//! |----------------|-------------|----------------|-----------------------------------|
//! | Global         | 1615        | 1              | Player vitals, statuses, perks,   |
//! |                |             |                | rebuild state, 6× card histograms |
//! | Hand cards     | 314         | MAX_HAND=12    | Cost, mechanics, statuses, flags, |
//! |                |             |                | identity one-hot                  |
//! | Enemies        | 54          | MAX_ENEMIES=4  | HP, intent, statuses, identity    |
//! | Action feats   | 652         | MAX_ACTIONS=64 | Action type, 2 card slots, target |
//!
//! Plus an action mask (MAX_ACTIONS) and counts (n_hand, n_enemies, n_actions)
//! so the model can distinguish real items from padding zeros.

use std::collections::HashMap;

use decker_engine::card::CardDef;
use decker_engine::combat::Action;
use decker_engine::enemy::IntentType;
use decker_engine::status::StatusType;

use decker_gauntlet::observation::{CardObs, EnemyObs, Observation};
use decker_gauntlet::{GauntletAction, GauntletRunner, PLAY_DECK_SIZE};

// ── Padding constants for fixed-size observation tensors ─────────────────

pub const MAX_HAND: usize = 12;
pub const MAX_ENEMIES: usize = 4;
pub const MAX_ACTIONS: usize = 64;

// ── Card vocabulary ──────────────────────────────────────────────────────

/// Ordered list of all known card IDs for embedding lookup.
/// Unknown cards map to index 0 (the "unknown" slot).
const CARD_VOCAB: &[&str] = &[
    "<unk>",
    "strike",
    "defend",
    "wound",
    "barbs",
    "heavy_strike",
    "measured_strike",
    "second_wind",
    "energy_surge",
    "extra_attack",
    "rally",
    "iron_guard",
    "shield_bash",
    "brace",
    "unbreakable",
    "intimidating_blow",
    "reckless_attack",
    "sundering_blow",
    "reaving_strike",
    "sizing_up",
    "precise_cut",
    "expose_weakness",
    "calculated_strike",
    "read_and_react",
    "flurry_of_cuts",
    "power_strike",
    "quick_slash",
    "vicious_strike",
    "double_strike",
    "brace_for_impact",
    "hold_the_line",
    "taunt",
    "focus",
    "martial_ascendancy",
    "iron_fortress",
    "titans_fury",
    "coup_de_grace",
    // New dueling cards (Phase 1a+)
    "deft_strike",
    "riposte",
    "feint",
    "patient_strike",
    "perfect_rhythm",
    "perfect_read",
    // New dueling progression
    "adrenaline_rush",
    "exploit_opening",
    "momentum",
    // Race cards
    "improvise",
    "fae_ancestry",
    "blood_price",
    "stonewall",
    "flash_bang",
    "nimble_dodge",
    "savage_charge",
    "shiv",
    "dragon_breath",
    "pounce",
    // Background cards
    "battle_discipline",
    "studied_analysis",
    "minor_blessing",
    "dirty_trick",
    "commanding_presence",
    "survival_instinct",
    "improvised_weapon",
    "sea_legs",
    "backstab",
    "inner_focus",
    "resourceful",
    "dazzle",
    // Defense: Bulwark synergy
    "fortify",
    "entrench",
    "stalwart_defense",
    "shield_wall",
    "aegis_eternal",
    // Defense: Reprisal synergy
    "spiked_armor",
    "retribution",
    "thorned_carapace",
    "barbed_bulwark",
    "wrath_of_thorns",
    // Two-Handed: Crusher synergy
    "brutal_swing",
    "savage_presence",
    "demolish",
    "wrath_of_the_giant",
    "executioners_blow",
    // Two-Handed: Berserker synergy
    "blood_frenzy",
    "raging_blow",
    "berserk_roar",
    "unleash_fury",
    "deathwish",
    // Barbarian class + auto-grants
    "bloodlust",
    "raging_strike",
    "totem_guard",
    "frenzied_slash",
    "battle_fury",
    "thick_skin",
    "primal_surge",
    "undying_rage",
    // Berserker: Bloodrage
    "blood_offering",
    "fury_unleashed",
    "bloodbath",
    "pain_is_power",
    "berserker_deathwish",
    // Berserker: Rampage
    "wild_swing",
    "berserk_flurry",
    "savage_momentum",
    "unstoppable",
    "rampage",
    // Totem Warrior: Spirit Shield
    "spirit_ward",
    "ancestral_shield",
    "warding_totem_card",
    "fortified_rage",
    "unbreaking_spirit",
    // Totem Warrior: Ancestral
    "war_cry",
    "spirit_mend",
    "vengeful_ancestors",
    "totem_of_renewal",
    "ancestors_embrace",
    // Frenzy: Adrenaline
    "desperate_strike",
    "adrenaline_spike",
    "feral_instinct",
    "berserkers_trance_card",
    "deaths_door",
    // Frenzy: Overwhelm
    "flailing_strike",
    "frenzy_card",
    "whirlwind_fury",
    "one_thousand_cuts",
    // Monk class + auto-grants
    "inner_peace",
    "centered_strike",
    "swift_footwork",
    "ki_surge",
    "meditation",
    // Monk shared pool (level 1)
    "ki_focus",
    "ki_strike",
    "ki_guard",
    // Open Hand: Flurry
    "rapid_strikes",
    "palm_strike",
    "whirlwind_kick",
    "hundred_fists",
    "endless_barrage",
    // Open Hand: Ki Burst
    "focused_strike",
    "ki_shield",
    "ki_channeling",
    "quivering_palm",
    "transcendence",
    // Way of Shadow: Pressure Point
    "nerve_strike",
    "pressure_point_strike",
    "dim_mak",
    "crippling_blow",
    "weakening_aura",
    "death_touch",
    // Way of Shadow: Evasion
    "shadow_step",
    "deflecting_palm",
    "counter_stance",
    "phantom_form",
    "shadow_kill",
    // Iron Fist: Stance Flow
    "stance_shift",
    "tigers_fury",
    "cranes_wing",
    "flowing_water",
    "monk_dragons_breath",
    "perfect_harmony",
    // Iron Fist: Iron Will
    "iron_skin",
    "ki_barrier",
    "stone_body",
    "diamond_soul",
    "fist_of_north_star",
    // Warlock class + shared
    "eldritch_blast",
    "hex",
    "dark_bargain",
    "maledict",
    "pact_barrier",
    "siphon_bolt",
    // Infernal: Infernal Burn
    "hellbrand",
    "cinder_feast",
    "balefire_cataclysm",
    // Infernal: Infernal Sustain
    "blood_tithe",
    "feast_of_cinders",
    // Hexblade: Hexblade Duel
    "pact_blade",
    "soul_parry",
    "dusk_duel",
    // Hexblade: Hexblade Curse
    "brand_the_weak",
    "black_oath",
    // Void: Void Debuff
    "void_gaze",
    "starless_whisper",
    "entropy_field",
    // Void: Void Annihilation
    "oblivion_tide",
    "cosmic_extinction",
    // Paladin shared
    "holy_strike",
    "lay_on_hands",
    "prayer_of_valor",
    "consecration",
    "divine_bulwark",
    // Paladin: Devotion
    "warding_slash",
    "shield_of_faith",
    "bastion_drive",
    "holy_bastion",
    // Paladin: Vengeance
    "avenging_strike",
    "blade_of_wrath",
    "divine_judgment",
    "zealous_rush",
    // Paladin: Radiance
    "radiant_burst",
    "beacon_of_light",
    "solar_flare",
    "dawns_blessing",
    // Paladin: Capstone
    "avatar_of_radiance",
    // Rogue class card
    "tricks_of_the_trade",
    // Rogue shared auto-grants
    "sneak_attack",
    "cunning_action",
    "rogue_evasion",
    "preparation",
    // Assassin synergy
    "assassinate",
    "rogue_shadow_step",
    "death_mark",
    "killing_blow",
    // Pirate synergy
    "cutlass_flurry",
    "grappling_hook",
    "dirty_fighting",
    "broadside",
    // Trickster synergy
    "smoke_bomb",
    "misdirection",
    "fan_of_knives",
    "ace_in_the_hole",
    // Druid: form-entry cantrips
    "wild_shape_bear",
    "wild_shape_eagle",
    "wild_shape_wolf",
    // Druid: shared
    "natures_wrath",
    "bark_skin",
    "rejuvenation",
    // Druid: Bear synergy
    "bear_maul",
    "thick_hide",
    "ursine_charge",
    "primal_roar",
    // Druid: Eagle synergy
    "eagle_dive",
    "swooping_strike",
    "wind_rider",
    "tempest_talons",
    // Druid: Wolf synergy
    "pack_tactics",
    "howl",
    "coordinated_strike",
    "alphas_command",
    // Druid: concentration
    "moonbeam",
    "entangle",
    // Druid: capstone
    "archdruid",
    // Wizard shared
    "fire_bolt",
    "mage_armor",
    "arcane_intellect",
    "shield_spell",
    // Wizard themed basics
    "ember_bolt",
    "arcane_ward",
    "prescience",
    // Evocation
    "fireball",
    "scorching_ray",
    "lightning_bolt",
    "flame_shield",
    // Abjuration
    "counterspell",
    "ward",
    "dispel",
    "globe_of_protection",
    // Divination
    "foresight",
    "scrying",
    "portent",
    "time_stop",
    // Wizard capstone
    "wish",
];

pub const VOCAB_SIZE: usize = 262;

#[allow(dead_code)]
fn card_vocab_index(card_id: &str) -> usize {
    CARD_VOCAB.iter().position(|&c| c == card_id).unwrap_or(0)
}

// ── Dynamic feature configuration ──────────────────────────────────────

/// Configuration for feature encoding with a dynamic card vocabulary.
///
/// When no synergy filter is active, this uses the full CARD_VOCAB (53 cards).
/// With a filter, it shrinks to only the cards that can appear in the run,
/// reducing observation dimensions by ~40%.
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    /// Ordered card vocab for this config. Index 0 is always "<unk>".
    pub card_vocab: Vec<String>,
    pub vocab_size: usize,
    pub card_feat_dim: usize,
    pub action_feat_dim: usize,
    pub global_dim: usize,
}

impl FeatureConfig {
    /// Build a FeatureConfig from a list of active card IDs.
    /// The vocab will be: ["<unk>"] + sorted active_card_ids.
    pub fn new(active_card_ids: &[String]) -> Self {
        let mut vocab: Vec<String> = vec!["<unk>".to_string()];
        let mut sorted = active_card_ids.to_vec();
        sorted.sort();
        sorted.dedup();
        vocab.extend(sorted);
        let vocab_size = vocab.len();
        let card_feat_dim = 52 + vocab_size;
        let action_feat_dim = ACTION_TYPES + card_feat_dim * 2 + 6 + 8;
        let global_dim = 43 + vocab_size * 6;
        Self {
            card_vocab: vocab,
            vocab_size,
            card_feat_dim,
            action_feat_dim,
            global_dim,
        }
    }

    /// Build the default (unfiltered) config from the static CARD_VOCAB.
    #[allow(dead_code)]
    pub fn default_config() -> Self {
        let vocab: Vec<String> = CARD_VOCAB.iter().map(|&s| s.to_string()).collect();
        let vocab_size = vocab.len();
        Self {
            card_vocab: vocab,
            vocab_size,
            card_feat_dim: 52 + vocab_size,
            action_feat_dim: ACTION_TYPES + (52 + vocab_size) * 2 + 6 + 8,
            global_dim: 43 + vocab_size * 6,
        }
    }

    /// Look up a card's index in this config's vocab. Returns 0 (<unk>) if not found.
    pub fn card_index(&self, card_id: &str) -> usize {
        self.card_vocab.iter().position(|c| c == card_id).unwrap_or(0)
    }

    /// Build a card histogram over this config's vocab.
    fn card_histogram(&self, card_ids: &[String]) -> Vec<f64> {
        let mut counts = vec![0.0_f64; self.vocab_size];
        for id in card_ids {
            let idx = self.card_index(id);
            counts[idx] += 1.0;
        }
        for c in counts.iter_mut() {
            *c /= 5.0;
        }
        counts
    }

    /// Card features for a single card observation.
    pub fn card_features(&self, card: &CardObs) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.card_feat_dim);

        v.push(card.cost as f64 / 3.0);
        v.push(if card.playable { 1.0 } else { 0.0 });
        v.push(card.total_damage as f64 / 15.0);
        v.push(card.total_block as f64 / 15.0);
        v.push(if card.exhaust { 1.0 } else { 0.0 });
        v.push(card.draw_count as f64 / 3.0);
        v.push(card.energy_gain as f64 / 3.0);
        v.extend(status_vec(&card.enemy_statuses));
        v.extend(status_vec(&card.self_statuses));
        v.push(if card.targets_all { 1.0 } else { 0.0 });
        v.push(card.hp_change as f64 / 10.0);
        v.push(if card.innate { 1.0 } else { 0.0 });
        v.push(if card.recycle { 1.0 } else { 0.0 });
        v.push(if card.has_conditional { 1.0 } else { 0.0 });

        let idx = self.card_index(&card.card_id);
        for i in 0..self.vocab_size {
            v.push(if i == idx { 1.0 } else { 0.0 });
        }

        debug_assert_eq!(v.len(), self.card_feat_dim);
        v
    }

    /// Global features using this config's vocab dimensions.
    pub fn global_features(&self, obs: &Observation) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.global_dim);

        let max_hp = obs.player_max_hp.max(1) as f64;
        v.push(obs.player_hp as f64 / max_hp);
        v.push(obs.player_block as f64 / 20.0);
        let max_e = obs.player_max_energy.max(1) as f64;
        v.push(obs.player_energy as f64 / max_e);
        v.push(obs.turn as f64 / 10.0);
        v.push(obs.fights_won as f64 / 50.0);
        v.push(obs.draw_pile_size as f64 / 20.0);
        v.push(obs.discard_pile_size as f64 / 20.0);
        v.push(obs.play_deck_size as f64 / 16.0);

        use decker_gauntlet::observation::ObservedPhase;
        let phase_idx = match &obs.phase_type {
            ObservedPhase::Combat => 0,
            ObservedPhase::Reward => 1,
            ObservedPhase::CollectionOverflow => 2,
            ObservedPhase::InnateChoice => 3,
            ObservedPhase::DeckSwap => 4,
            ObservedPhase::DeckRebuild => 5,
            ObservedPhase::GameOver => 6,
        };
        for i in 0..7usize {
            v.push(if i == phase_idx { 1.0 } else { 0.0 });
        }

        v.extend(status_vec(&obs.player_statuses));
        v.push(obs.player_level as f64 / 12.0);
        v.push(obs.bonus_draw as f64 / 3.0);
        v.push(obs.bonus_energy as f64 / 3.0);
        v.push(obs.collection_size as f64 / 40.0);
        v.push(obs.block_floor as f64 / 20.0);
        v.push(obs.retain_block_cap as f64 / 20.0);
        v.push(obs.rebuild_slot_index as f64 / PLAY_DECK_SIZE as f64);
        v.push(
            (PLAY_DECK_SIZE.saturating_sub(obs.rebuild_partial_deck_card_ids.len())) as f64
                / PLAY_DECK_SIZE as f64,
        );

        if obs.in_combat {
            v.extend(self.card_histogram(&obs.draw_pile_card_ids));
            v.extend(self.card_histogram(&obs.hand_card_ids));
            v.extend(self.card_histogram(&obs.discard_pile_card_ids));
            v.extend(self.card_histogram(&obs.exhaust_pile_card_ids));
            v.extend(self.card_histogram(&obs.innate_card_ids));
            v.extend(vec![0.0; self.vocab_size]);
        } else {
            v.extend(self.card_histogram(&obs.play_deck_card_ids));
            v.extend(self.card_histogram(&obs.collection_card_ids));
            let choice_ids: Vec<String> = if obs.phase_type == ObservedPhase::Reward {
                obs.reward_cards.iter().map(|c| c.card_id.clone()).collect()
            } else {
                obs.choice_cards.iter().map(|c| c.card_id.clone()).collect()
            };
            v.extend(self.card_histogram(&choice_ids));
            v.extend(self.card_histogram(&obs.rebuild_partial_deck_card_ids));
            v.extend(self.card_histogram(&obs.innate_card_ids));
            v.extend(self.card_histogram(&obs.rebuild_remaining_card_ids));
        }

        debug_assert_eq!(v.len(), self.global_dim);
        v
    }

    /// Action features for a single action using this config's dimensions.
    pub fn action_features(
        &self,
        action: &GauntletAction,
        obs: &Observation,
        content: &decker_engine::content_tables::ContentTables,
    ) -> Vec<f64> {
        let mut v = vec![0.0; self.action_feat_dim];

        let slot_a_off = ACTION_TYPES;
        let slot_b_off = slot_a_off + self.card_feat_dim;
        let tgt_off = slot_b_off + self.card_feat_dim;
        let ctx_off = tgt_off + 6;

        let fc = self;
        /// Helper: write card features into a slot.
        fn write_card_slot(v: &mut [f64], off: usize, card: &CardObs, fc: &FeatureConfig) {
            let cf = fc.card_features(card);
            v[off..off + fc.card_feat_dim].copy_from_slice(&cf);
        }

        fn write_def_slot(
            v: &mut [f64],
            off: usize,
            card_id: &str,
            content: &decker_engine::content_tables::ContentTables,
            fc: &FeatureConfig,
        ) {
            if let Some(def) = content.card_defs.get(card_id) {
                let card_obs = card_obs_from_def(card_id, def);
                let cf = fc.card_features(&card_obs);
                v[off..off + fc.card_feat_dim].copy_from_slice(&cf);
            }
        }

        match action {
            GauntletAction::CombatAction(Action::PlayCard { hand_index, target }) => {
                v[0] = 1.0;
                if let Some(card) = obs.hand.get(*hand_index) {
                    write_card_slot(&mut v, slot_a_off, card, fc);
                }
                if let Some(tidx) = target {
                    if let Some(enemy) = obs.enemies.get(*tidx) {
                        let ef = enemy_features_single(enemy);
                        v[tgt_off] = ef[0];     // HP fraction
                        v[tgt_off + 1] = ef[2]; // block
                        v[tgt_off + 2] = ef[3]; // intent_index
                        v[tgt_off + 3] = ef[4]; // alive
                        v[tgt_off + 4] = ef[11]; // intent primary magnitude
                        if let Some(card) = obs.hand.get(*hand_index) {
                            let dmg = (card.total_damage - enemy.block).max(0) as f64;
                            let hp = enemy.hp.max(1) as f64;
                            v[tgt_off + 5] = (dmg / hp).min(2.0);
                        }
                    }
                }
            }
            GauntletAction::CombatAction(Action::EndTurn) => { v[1] = 1.0; }
            GauntletAction::CombatAction(Action::DiscardCard { hand_index }) => {
                v[2] = 1.0;
                if let Some(card) = obs.hand.get(*hand_index) {
                    write_card_slot(&mut v, slot_b_off, card, fc);
                }
            }
            GauntletAction::PickReward(idx) => {
                v[3] = 1.0;
                if let Some(card) = obs.reward_cards.get(*idx) {
                    write_card_slot(&mut v, slot_a_off, card, fc);
                }
            }
            GauntletAction::SkipReward => { v[4] = 1.0; }
            GauntletAction::RemoveCollectionCard(idx) => {
                v[5] = 1.0;
                if let Some(card) = obs.choice_cards.get(*idx) {
                    write_card_slot(&mut v, slot_b_off, card, fc);
                }
            }
            GauntletAction::ChooseInnate(idx) => {
                v[6] = 1.0;
                if let Some(card_id) = obs.play_deck_card_ids.get(*idx) {
                    write_def_slot(&mut v, slot_a_off, card_id, content, fc);
                }
            }
            GauntletAction::SwapIntoDeck(idx) => {
                v[7] = 1.0;
                if let Some(card) = &obs.acquired_card {
                    write_card_slot(&mut v, slot_a_off, card, fc);
                }
                if let Some(card_id) = obs.play_deck_card_ids.get(*idx) {
                    write_def_slot(&mut v, slot_b_off, card_id, content, fc);
                }
            }
            GauntletAction::SkipSwap => { v[8] = 1.0; }
            GauntletAction::ChooseDeckSlotCard(choice_idx) => {
                v[9] = 1.0;
                if let Some(card) = obs.choice_cards.get(*choice_idx) {
                    write_card_slot(&mut v, slot_a_off, card, fc);
                }
                if let Some(card_id) = obs.play_deck_card_ids.get(obs.rebuild_slot_index) {
                    write_def_slot(&mut v, slot_b_off, card_id, content, fc);
                }
            }
        }

        let max_hp = obs.player_max_hp.max(1) as f64;
        v[ctx_off] = obs.player_hp as f64 / max_hp;
        v[ctx_off + 1] = obs.player_energy as f64 / obs.player_max_energy.max(1) as f64;
        v[ctx_off + 2] = obs.player_block as f64 / 20.0;
        let incoming: f64 = obs.enemies.iter()
            .filter(|e| e.alive)
            .map(expected_incoming_damage)
            .sum();
        v[ctx_off + 3] = incoming / 30.0;
        let deficit = (incoming - obs.player_block as f64).max(0.0);
        v[ctx_off + 4] = deficit / 30.0;
        v[ctx_off + 5] = obs.enemies.iter().filter(|e| e.alive).count() as f64 / 4.0;
        v[ctx_off + 6] = obs.rebuild_slot_index as f64 / PLAY_DECK_SIZE as f64;
        v[ctx_off + 7] = (PLAY_DECK_SIZE.saturating_sub(obs.rebuild_partial_deck_card_ids.len()))
            as f64 / PLAY_DECK_SIZE as f64;

        v
    }

    /// All action features for a runner's current state.
    pub fn all_action_features(&self, runner: &GauntletRunner) -> Vec<Vec<f64>> {
        let obs = runner.observe();
        let actions = runner.legal_actions();
        actions.iter().map(|a| self.action_features(a, &obs, &runner.content)).collect()
    }

    /// Hand card features.
    pub fn hand_features(&self, obs: &Observation) -> Vec<Vec<f64>> {
        obs.hand.iter().map(|c| self.card_features(c)).collect()
    }
}

// ── Enemy vocabulary ────────────────────────────────────────────────────

/// Ordered list of all known enemy IDs for identity encoding.
const ENEMY_VOCAB: &[&str] = &[
    "<unk>",
    // Minions
    "goblin_grunt",
    "goblin_archer",
    "goblin_shaman",
    "giant_rat",
    "scavenger",
    "fire_beetle",
    "animated_shield",
    "sporecap",
    "skeleton",
    "blood_moth",
    "bandit_thug",
    // Standard
    "slime",
    "zombie",
    "plague_rat",
    "cave_spider",
    "giant_centipede",
    "bandit_archer",
    "imp",
    "shadow_wisp",
    "bone_sentinel",
    "harpy",
    "specter",
    "rust_crawler",
    "blightcap",
    "wolf",
    "boar",
    "cult_acolyte",
    "wight",
    "sellsword",
    "young_drake",
    "animated_armor",
    // Tough
    "ogre_thug",
    "stone_golem",
    "cave_stalker",
    "husk_whisperer",
    "troll",
    "bone_commander",
    "cult_channeler",
    "cult_invoker",
    "dark_priest",
    "corpse_knight",
    "sellsword_captain",
    // Elite
    "flamewrath",
    "wyvern",
    "necromancer",
    "high_cultist",
    "iron_golem",
    // Bosses
    "ogre_warchief",
    "pale_warden",
    "dragon_wyrm",
];

pub const ENEMY_VOCAB_SIZE: usize = 51;

fn enemy_vocab_index(enemy_id: &str) -> usize {
    ENEMY_VOCAB.iter().position(|&e| e == enemy_id).unwrap_or(0)
}

// ── Status helpers ───────────────────────────────────────────────────────

const STATUS_ORDER: [StatusType; 20] = [
    StatusType::Threatened,
    StatusType::Weakened,
    StatusType::Empowered,
    StatusType::Bleeding,
    StatusType::Barbed,
    StatusType::Mending,
    StatusType::Frightened,
    StatusType::Marked,
    StatusType::Armored,
    StatusType::BlockRetention,
    StatusType::Momentum,
    StatusType::SavageBlows,
    StatusType::Hexed,
    StatusType::Artifice,
    StatusType::WildShapeBear,
    StatusType::WildShapeEagle,
    StatusType::WildShapeWolf,
    StatusType::ArcaneCharge,
    StatusType::Fortified,
    StatusType::SanctifiedShield,
];

fn status_vec(statuses: &[(StatusType, i32)]) -> Vec<f64> {
    let map: HashMap<StatusType, i32> = statuses.iter().cloned().collect();
    STATUS_ORDER
        .iter()
        .map(|st| map.get(st).copied().unwrap_or(0) as f64 / 5.0)
        .collect()
}

// ── Card histogram helper ────────────────────────────────────────────────

/// Build a histogram of card counts over CARD_VOCAB, normalized by dividing by 5.
#[allow(dead_code)]
fn card_histogram(card_ids: &[String]) -> Vec<f64> {
    let mut counts = vec![0.0_f64; VOCAB_SIZE];
    for id in card_ids {
        let idx = card_vocab_index(id);
        counts[idx] += 1.0;
    }
    for c in counts.iter_mut() {
        *c /= 5.0;
    }
    counts
}

fn expected_incoming_damage(enemy: &EnemyObs) -> f64 {
    match &enemy.intent {
        Some(IntentType::Attack(n)) => *n as f64,
        Some(IntentType::AttackDefend(n, _)) => *n as f64,
        _ => 0.0,
    }
}

// ── Global features ──────────────────────────────────────────────────────

/// Number of global features: 38 base + 6 * VOCAB_SIZE (card histograms).
///
/// Layout (smaller with synergy/race filter):
///   [0]      HP fraction
///   [1]      block / 20
///   [2]      energy fraction
///   [3]      turn / 10
///   [4]      fights_won / 50
///   [5]      draw_pile_size / 20
///   [6]      discard_pile_size / 20
///   [7]      play_deck_size / 16
///   [8-14]   phase one-hot (Combat, Reward, CollectionOverflow,
///            InnateChoice, DeckSwap, DeckRebuild, GameOver)
///   [15-34]  player statuses (20)
///   [35]     player_level / 12
///   [36]     bonus_draw / 3
///   [37]     bonus_energy / 3
///   [38]     collection_size / 40
///   [39]     block_floor / 20
///   [40]     retain_block_cap / 20
///   [41]     rebuild_slot_index / 16
///   [42]     rebuild_remaining_slots / 16
///   [43-N]   card histograms (6 x VOCAB_SIZE)
///            Combat: draw pile, hand, discard, exhaust, innate, zeros
///            Non-combat: play deck, collection, rewards-or-choice-set,
///            partial rebuild deck, innate, remaining rebuild inventory
pub const GLOBAL_DIM: usize = 43 + VOCAB_SIZE * 6;

#[allow(dead_code)]
pub fn global_features(obs: &Observation) -> Vec<f64> {
    let mut v = Vec::with_capacity(GLOBAL_DIM);

    // Player vitals (normalized).
    let max_hp = obs.player_max_hp.max(1) as f64;
    v.push(obs.player_hp as f64 / max_hp); // HP fraction
    v.push(obs.player_block as f64 / 20.0); // block (rough norm)
    let max_e = obs.player_max_energy.max(1) as f64;
    v.push(obs.player_energy as f64 / max_e); // energy fraction

    // Turn and progress.
    v.push(obs.turn as f64 / 10.0);
    v.push(obs.fights_won as f64 / 50.0);

    // Pile sizes.
    v.push(obs.draw_pile_size as f64 / 20.0);
    v.push(obs.discard_pile_size as f64 / 20.0);
    v.push(obs.play_deck_size as f64 / 16.0);

    // Phase one-hot (7 values: one per ObservedPhase variant).
    use decker_gauntlet::observation::ObservedPhase;
    let phase_idx = match &obs.phase_type {
        ObservedPhase::Combat => 0,
        ObservedPhase::Reward => 1,
        ObservedPhase::CollectionOverflow => 2,
        ObservedPhase::InnateChoice => 3,
        ObservedPhase::DeckSwap => 4,
        ObservedPhase::DeckRebuild => 5,
        ObservedPhase::GameOver => 6,
    };
    for i in 0..7usize {
        v.push(if i == phase_idx { 1.0 } else { 0.0 });
    }

    // Player statuses (20 values, normalized).
    v.extend(status_vec(&obs.player_statuses));

    // Player level and perk/collection info.
    v.push(obs.player_level as f64 / 12.0);
    v.push(obs.bonus_draw as f64 / 3.0);
    v.push(obs.bonus_energy as f64 / 3.0);
    v.push(obs.collection_size as f64 / 40.0);
    v.push(obs.block_floor as f64 / 20.0);
    v.push(obs.retain_block_cap as f64 / 20.0);
    v.push(obs.rebuild_slot_index as f64 / PLAY_DECK_SIZE as f64);
    v.push(
        (PLAY_DECK_SIZE.saturating_sub(obs.rebuild_partial_deck_card_ids.len())) as f64
            / PLAY_DECK_SIZE as f64,
    );

    // Card histograms (6 x VOCAB_SIZE features).
    if obs.in_combat {
        v.extend(card_histogram(&obs.draw_pile_card_ids));
        v.extend(card_histogram(&obs.hand_card_ids));
        v.extend(card_histogram(&obs.discard_pile_card_ids));
        v.extend(card_histogram(&obs.exhaust_pile_card_ids));
        v.extend(card_histogram(&obs.innate_card_ids));
        v.extend(vec![0.0; VOCAB_SIZE]);
    } else {
        v.extend(card_histogram(&obs.play_deck_card_ids));
        v.extend(card_histogram(&obs.collection_card_ids));
        let choice_ids: Vec<String> = if obs.phase_type == ObservedPhase::Reward {
            obs.reward_cards.iter().map(|c| c.card_id.clone()).collect()
        } else {
            obs.choice_cards.iter().map(|c| c.card_id.clone()).collect()
        };
        v.extend(card_histogram(&choice_ids));
        v.extend(card_histogram(&obs.rebuild_partial_deck_card_ids));
        v.extend(card_histogram(&obs.innate_card_ids));
        v.extend(card_histogram(&obs.rebuild_remaining_card_ids));
    }

    debug_assert_eq!(v.len(), GLOBAL_DIM);
    v
}

// ── Per-card features ────────────────────────────────────────────────────

/// Number of features per card.
///
/// Layout (74 features):
///   [0]      cost / 3
///   [1]      playable
///   [2]      total_damage / 15
///   [3]      total_block / 15
///   [4]      exhaust
///   [5]      draw_count / 3
///   [6]      energy_gain / 3
///   [7-26]   enemy statuses per STATUS_ORDER (20 values, stacks/5)
///   [27-46]  self statuses per STATUS_ORDER (20 values, stacks/5)
///   [47]     targets_all (AoE flag)
///   [48]     hp_change / 10
///   [49]     innate
///   [50]     recycle
///   [51]     has_conditional
///   [52-N]   card identity one-hot (VOCAB_SIZE)
pub const CARD_FEAT_DIM: usize = 52 + VOCAB_SIZE;

#[allow(dead_code)]
pub fn card_features_from_obs(card: &CardObs) -> Vec<f64> {
    let mut v = Vec::with_capacity(CARD_FEAT_DIM);

    // Core mechanics.
    v.push(card.cost as f64 / 3.0);
    v.push(if card.playable { 1.0 } else { 0.0 });
    v.push(card.total_damage as f64 / 15.0);
    v.push(card.total_block as f64 / 15.0);
    v.push(if card.exhaust { 1.0 } else { 0.0 });
    v.push(card.draw_count as f64 / 3.0);
    v.push(card.energy_gain as f64 / 3.0);

    // Per-status stacks applied to enemies (20 values).
    v.extend(status_vec(&card.enemy_statuses));

    // Per-status stacks applied to self (20 values).
    v.extend(status_vec(&card.self_statuses));

    // More mechanics.
    v.push(if card.targets_all { 1.0 } else { 0.0 });
    v.push(card.hp_change as f64 / 10.0);

    // Card flags.
    v.push(if card.innate { 1.0 } else { 0.0 });
    v.push(if card.recycle { 1.0 } else { 0.0 });
    v.push(if card.has_conditional { 1.0 } else { 0.0 });

    // Card identity one-hot (VOCAB_SIZE values).
    let idx = card_vocab_index(&card.card_id);
    for i in 0..VOCAB_SIZE {
        v.push(if i == idx { 1.0 } else { 0.0 });
    }

    debug_assert_eq!(v.len(), CARD_FEAT_DIM);
    v
}

/// All hand cards as a flat Vec<Vec<f64>>.
#[allow(dead_code)]
pub fn hand_features(obs: &Observation) -> Vec<Vec<f64>> {
    obs.hand.iter().map(card_features_from_obs).collect()
}

// ── Per-enemy features ───────────────────────────────────────────────────

/// Number of features per enemy.
///
/// Layout (40 features):
///   [0]     hp / max_hp
///   [1]     hp / 100
///   [2]     block / 20
///   [3]     intent_index / 4
///   [4]     alive
///   [7-12]  intent type one-hot (Attack, Defend, Buff, Debuff, AttackDefend, BuffAllies)
///   [13]    intent primary magnitude / 20
///   [14]    intent secondary magnitude / 20 (AttackDefend defend value)
///   [15-34] intent status one-hot (which status for Buff/Debuff/BuffAllies intents, 20 values)
///   [35-54] enemy statuses per STATUS_ORDER (20 values, stacks/5)
///   [55]    enemy identity (vocab_index / ENEMY_VOCAB_SIZE)
pub const ENEMY_FEAT_DIM: usize = 54;

pub fn enemy_features_single(e: &EnemyObs) -> Vec<f64> {
    let max_hp = e.max_hp.max(1) as f64;
    let mut v = vec![
        e.hp as f64 / max_hp,
        e.hp as f64 / 100.0,
        e.block as f64 / 20.0,
        e.intent_index as f64 / 4.0,
        if e.alive { 1.0 } else { 0.0 },
    ];

    // Intent type one-hot (6 values).
    let (ia, id, ib, idb, iad, iba) = match &e.intent {
        Some(IntentType::Attack(_)) => (1.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        Some(IntentType::Defend(_)) => (0.0, 1.0, 0.0, 0.0, 0.0, 0.0),
        Some(IntentType::Buff(..)) => (0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
        Some(IntentType::Debuff(..)) => (0.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        Some(IntentType::AttackDefend(..)) => (0.0, 0.0, 0.0, 0.0, 1.0, 0.0),
        Some(IntentType::BuffAllies(..)) => (0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
        None => (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    };
    v.extend_from_slice(&[ia, id, ib, idb, iad, iba]);

    // Intent primary magnitude.
    let intent_primary = match &e.intent {
        Some(IntentType::Attack(n)) => *n as f64,
        Some(IntentType::Defend(n)) => *n as f64,
        Some(IntentType::Buff(_, n)) => *n as f64,
        Some(IntentType::Debuff(_, n)) => *n as f64,
        Some(IntentType::AttackDefend(a, _)) => *a as f64,
        Some(IntentType::BuffAllies(_, n)) => *n as f64,
        None => 0.0,
    };
    v.push(intent_primary / 20.0);

    // Intent secondary magnitude (defend component of AttackDefend).
    let intent_secondary = match &e.intent {
        Some(IntentType::AttackDefend(_, d)) => *d as f64,
        _ => 0.0,
    };
    v.push(intent_secondary / 20.0);

    // Intent status one-hot (15 values): which status for Buff/Debuff/BuffAllies.
    let intent_status = match &e.intent {
        Some(IntentType::Buff(st, _)) => Some(st),
        Some(IntentType::Debuff(st, _)) => Some(st),
        Some(IntentType::BuffAllies(st, _)) => Some(st),
        _ => None,
    };
    for st in &STATUS_ORDER {
        v.push(if intent_status == Some(st) { 1.0 } else { 0.0 });
    }

    // All enemy statuses (17 values).
    v.extend(status_vec(&e.statuses));

    // Enemy identity.
    v.push(enemy_vocab_index(&e.enemy_id) as f64 / ENEMY_VOCAB_SIZE as f64);

    debug_assert_eq!(v.len(), ENEMY_FEAT_DIM);
    v
}

pub fn all_enemy_features(obs: &Observation) -> Vec<Vec<f64>> {
    obs.enemies.iter().map(enemy_features_single).collect()
}

// ── Per-action features ──────────────────────────────────────────────────

/// Number of action type categories.
const ACTION_TYPES: usize = 10;

/// Number of features per action.
///
/// Layout (172 features):
///   [0-9]     Action type one-hot (10: PlayCard, EndTurn, Discard, PickReward,
///             SkipReward, RemoveCollectionCard, ChooseInnate,
///             SwapIntoDeck, SkipSwap, ChooseDeckSlotCard)
///   [10-83]   Slot A: card gained/played/assigned (CARD_FEAT_DIM=74)
///   [84-157]  Slot B: card lost/replaced (CARD_FEAT_DIM=74)
///   [158-163] Target enemy (6)
///   [164-171] Context (8)
pub const ACTION_FEAT_DIM: usize = ACTION_TYPES + CARD_FEAT_DIM * 2 + 6 + 8; // 652

/// Build a CardObs from a CardDef (for non-hand contexts like deck removal).
fn card_obs_from_def(card_id: &str, def: &CardDef) -> CardObs {
    CardObs {
        card_id: card_id.to_string(),
        name: def.name.clone(),
        cost: def.cost,
        tags: def.tags.clone(),
        description: def.description.clone(),
        playable: false,
        total_damage: total_card_damage(def),
        total_block: total_card_block(def),
        exhaust: def.exhaust,
        draw_count: card_draw_count(def),
        energy_gain: card_energy_gain(def),
        enemy_statuses: card_enemy_statuses(def),
        self_statuses: card_self_statuses(def),
        targets_all: card_targets_all(def),
        hp_change: card_hp_change(def),
        innate: def.innate,
        recycle: def.recycle,
        has_conditional: card_has_conditional(def),
        concentration: def.concentration,
    }
}

/// Compute a semantic feature vector for a single GauntletAction.
///
/// Two card slots: Slot A = card being gained/played, Slot B = card being lost/replaced.
#[allow(dead_code)]
fn action_features_single(
    action: &GauntletAction,
    obs: &Observation,
    content: &decker_engine::content_tables::ContentTables,
) -> Vec<f64> {
    let mut v = vec![0.0; ACTION_FEAT_DIM];

    // Offsets into the feature vector.
    let slot_a_off = ACTION_TYPES; // card gained/played
    let slot_b_off = slot_a_off + CARD_FEAT_DIM; // card lost/replaced
    let tgt_off = slot_b_off + CARD_FEAT_DIM; // target enemy
    let ctx_off = tgt_off + 6; // context

    /// Helper: write card features into a slot.
    fn write_card_slot(v: &mut [f64], off: usize, card: &CardObs) {
        let cf = card_features_from_obs(card);
        v[off..off + CARD_FEAT_DIM].copy_from_slice(&cf);
    }

    /// Helper: build CardObs from def and write to slot.
    fn write_def_slot(
        v: &mut [f64],
        off: usize,
        card_id: &str,
        content: &decker_engine::content_tables::ContentTables,
    ) {
        if let Some(def) = content.card_defs.get(card_id) {
            let card_obs = card_obs_from_def(card_id, def);
            let cf = card_features_from_obs(&card_obs);
            v[off..off + CARD_FEAT_DIM].copy_from_slice(&cf);
        }
    }

    match action {
        GauntletAction::CombatAction(Action::PlayCard { hand_index, target }) => {
            v[0] = 1.0; // PlayCard
                        // Slot A: card played
            if let Some(card) = obs.hand.get(*hand_index) {
                write_card_slot(&mut v, slot_a_off, card);
            }
            // Target enemy features.
            if let Some(tidx) = target {
                if let Some(enemy) = obs.enemies.get(*tidx) {
                    let ef = enemy_features_single(enemy);
                    v[tgt_off] = ef[0]; // HP fraction
                    v[tgt_off + 1] = ef[2]; // block
                    v[tgt_off + 2] = ef[3]; // intent_index
                    v[tgt_off + 3] = ef[4]; // alive
                    v[tgt_off + 4] = ef[11]; // intent primary magnitude
                                             // Derived: post-block damage / HP ratio.
                    if let Some(card) = obs.hand.get(*hand_index) {
                        let dmg = (card.total_damage - enemy.block).max(0) as f64;
                        let hp = enemy.hp.max(1) as f64;
                        v[tgt_off + 5] = (dmg / hp).min(2.0);
                    }
                }
            }
        }
        GauntletAction::CombatAction(Action::EndTurn) => {
            v[1] = 1.0;
        }
        GauntletAction::CombatAction(Action::DiscardCard { hand_index }) => {
            v[2] = 1.0;
            // Slot B: card discarded (losing)
            if let Some(card) = obs.hand.get(*hand_index) {
                write_card_slot(&mut v, slot_b_off, card);
            }
        }
        GauntletAction::PickReward(idx) => {
            v[3] = 1.0;
            // Slot A: reward card gained
            if let Some(card) = obs.reward_cards.get(*idx) {
                write_card_slot(&mut v, slot_a_off, card);
            }
        }
        GauntletAction::SkipReward => {
            v[4] = 1.0;
        }
        GauntletAction::RemoveCollectionCard(idx) => {
            v[5] = 1.0;
            if let Some(card) = obs.choice_cards.get(*idx) {
                write_card_slot(&mut v, slot_b_off, card);
            }
        }
        GauntletAction::ChooseInnate(idx) => {
            v[6] = 1.0;
            // Slot A: deck card chosen as innate
            if let Some(card_id) = obs.play_deck_card_ids.get(*idx) {
                write_def_slot(&mut v, slot_a_off, card_id, content);
            }
        }
        GauntletAction::SwapIntoDeck(idx) => {
            v[7] = 1.0;
            // Slot A: acquired card being swapped in
            if let Some(card) = &obs.acquired_card {
                write_card_slot(&mut v, slot_a_off, card);
            }
            // Slot B: deck card being replaced
            if let Some(card_id) = obs.play_deck_card_ids.get(*idx) {
                write_def_slot(&mut v, slot_b_off, card_id, content);
            }
        }
        GauntletAction::SkipSwap => {
            v[8] = 1.0;
        }
        GauntletAction::ChooseDeckSlotCard(choice_idx) => {
            v[9] = 1.0;
            if let Some(card) = obs.choice_cards.get(*choice_idx) {
                write_card_slot(&mut v, slot_a_off, card);
            }
            if let Some(card_id) = obs.play_deck_card_ids.get(obs.rebuild_slot_index) {
                write_def_slot(&mut v, slot_b_off, card_id, content);
            }
        }
    }

    // Global context features (8 values).
    let max_hp = obs.player_max_hp.max(1) as f64;
    v[ctx_off] = obs.player_hp as f64 / max_hp;
    v[ctx_off + 1] = obs.player_energy as f64 / obs.player_max_energy.max(1) as f64;
    v[ctx_off + 2] = obs.player_block as f64 / 20.0;

    let incoming: f64 = obs
        .enemies
        .iter()
        .filter(|e| e.alive)
        .map(expected_incoming_damage)
        .sum();
    v[ctx_off + 3] = incoming / 30.0;

    let deficit = (incoming - obs.player_block as f64).max(0.0);
    v[ctx_off + 4] = deficit / 30.0;

    v[ctx_off + 5] = obs.enemies.iter().filter(|e| e.alive).count() as f64 / 4.0;
    v[ctx_off + 6] = obs.rebuild_slot_index as f64 / PLAY_DECK_SIZE as f64;
    v[ctx_off + 7] = (PLAY_DECK_SIZE.saturating_sub(obs.rebuild_partial_deck_card_ids.len()))
        as f64
        / PLAY_DECK_SIZE as f64;

    v
}

// ── Card helper functions for non-hand action contexts ───────────────────

/// Total base damage across all effects of a card.
fn total_card_damage(def: &CardDef) -> i32 {
    use decker_engine::card::Effect;
    def.effects
        .iter()
        .map(|e| match e {
            Effect::DealDamage { amount, .. } => *amount,
            Effect::DealDamageIfDamagedLastTurn { amount, .. } => *amount,
            Effect::DealDamageIfTargetDebuffed { amount, .. } => *amount,
            Effect::DealDamageIfEnemyBlocked { amount, .. } => *amount,
            Effect::DealDamageIfMarked { amount, .. } => *amount,
            Effect::DealDamagePerCardPlayed { per_card, .. } => *per_card,
            Effect::DealDamageEqualBlock { .. } => 0,
            Effect::DealDamagePerMarkedStack { per_stack, .. } => *per_stack,
            Effect::DealDamageScaledByMarked {
                base_damage,
                per_stack,
                ..
            } => base_damage + per_stack,
            Effect::DealDamagePerDebuffOnTarget { base, per_debuff, .. } => base + per_debuff,
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

/// Total base block across all effects of a card.
fn total_card_block(def: &CardDef) -> i32 {
    use decker_engine::card::Effect;
    def.effects
        .iter()
        .map(|e| match e {
            Effect::GainBlock { amount } => *amount,
            Effect::GainBlockIfDamagedLastTurn { amount } => *amount,
            Effect::BlockEqualMissingHp { max } => *max,
            Effect::GainBlockPerStatusStack { multiplier, .. } => *multiplier,
            Effect::GainBlockConditional { amount, .. } => *amount,
            Effect::GainBlockSpendKi { base, .. } => *base,
            Effect::GainBlockPerKiStack { multiplier } => *multiplier,
            Effect::GainBlockSpendSmite { base, .. } => *base,
            Effect::GainBlockWithFormBonus { base, .. } => *base,
            _ => 0,
        })
        .sum()
}

/// Total cards drawn by a card's effects (net of discards).
fn card_draw_count(def: &CardDef) -> i32 {
    use decker_engine::card::Effect;
    def.effects
        .iter()
        .map(|e| match e {
            Effect::DrawCards { count } => *count as i32,
            Effect::DrawAndDiscard { draw, discard } => *draw as i32 - *discard as i32,
            Effect::DrawPerEnemyWithStatus { .. } => 2,
            Effect::DrawCardsWithFormBonus { base_draw, .. } => *base_draw as i32,
            _ => 0,
        })
        .sum()
}

/// Total energy gained from a card's effects.
fn card_energy_gain(def: &CardDef) -> i32 {
    use decker_engine::card::Effect;
    def.effects
        .iter()
        .map(|e| match e {
            Effect::GainEnergy { amount } => *amount,
            _ => 0,
        })
        .sum()
}

/// Per-status stacks applied to enemies.
fn card_enemy_statuses(def: &CardDef) -> Vec<(StatusType, i32)> {
    use decker_engine::card::{Effect, Target};
    let mut map: HashMap<StatusType, i32> = HashMap::new();
    for e in &def.effects {
        match e {
            Effect::ApplyStatus {
                target: Target::SingleEnemy | Target::AllEnemies | Target::RandomEnemy,
                status,
                stacks,
            } => {
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

/// Per-status stacks applied to self (player).
fn card_self_statuses(def: &CardDef) -> Vec<(StatusType, i32)> {
    use decker_engine::card::{Effect, Target};
    let mut map: HashMap<StatusType, i32> = HashMap::new();
    for e in &def.effects {
        match e {
            Effect::ApplyStatus {
                target: Target::Player,
                status,
                stacks,
            } => {
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

/// Whether any effect targets all enemies (AoE).
fn card_targets_all(def: &CardDef) -> bool {
    use decker_engine::card::{Effect, Target};
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
            } | Effect::DealDamagePerPlayerStatus {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageSpendKi {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageSpendSmite {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageSpendArtifice {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageIfInWildShape {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageWithFormBonus {
                target: Target::AllEnemies,
                ..
            } | Effect::DealDamageIfPlayerHasStatus {
                target: Target::AllEnemies,
                ..
            }
        )
    })
}

/// Net HP change from a card (heal minus self-damage).
fn card_hp_change(def: &CardDef) -> i32 {
    use decker_engine::card::Effect;
    def.effects
        .iter()
        .map(|e| match e {
            Effect::Heal { amount } => *amount,
            Effect::LoseHp { amount } => -*amount,
            _ => 0,
        })
        .sum()
}

/// Whether the card has conditional effects (damage/block depends on game state).
fn card_has_conditional(def: &CardDef) -> bool {
    use decker_engine::card::Effect;
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
        )
    })
}

/// Compute action feature vectors for all legal actions.
#[allow(dead_code)]
pub fn all_action_features(runner: &GauntletRunner) -> Vec<Vec<f64>> {
    let obs = runner.observe();
    let actions = runner.legal_actions();
    actions
        .iter()
        .map(|a| action_features_single(a, &obs, &runner.content))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use decker_gauntlet::observation::ObservedPhase;

    /// Verify constant consistency.
    #[test]
    fn action_feat_dim_matches_layout() {
        assert_eq!(ACTION_FEAT_DIM, ACTION_TYPES + CARD_FEAT_DIM * 2 + 6 + 8);
        assert_eq!(ACTION_FEAT_DIM, 652);
    }

    #[test]
    fn global_dim_matches_layout() {
        assert_eq!(GLOBAL_DIM, 43 + VOCAB_SIZE * 6);
        assert_eq!(GLOBAL_DIM, 1615);
    }

    #[test]
    fn all_enemy_ids_are_in_vocab() {
        let defs = decker_content::enemies::all_enemy_defs();
        for id in defs.keys() {
            assert_ne!(
                enemy_vocab_index(id.as_str()),
                0,
                "{} should not map to <unk> in RL features",
                id
            );
        }
        assert_eq!(ENEMY_VOCAB.len(), ENEMY_VOCAB_SIZE);
    }

    /// Helper to build a minimal Observation for testing.
    fn dummy_obs() -> Observation {
        Observation {
            in_combat: false,
            in_reward: false,
            game_over: false,
            phase_type: ObservedPhase::DeckSwap,
            player_hp: 50,
            player_max_hp: 80,
            player_block: 0,
            player_energy: 3,
            player_max_energy: 3,
            player_statuses: vec![],
            player_level: 3,
            hand: vec![],
            enemies: vec![],
            draw_pile_size: 0,
            discard_pile_size: 0,
            turn: 0,
            block_floor: 0,
            retain_block_cap: 0,
            draw_pile_card_ids: vec![],
            hand_card_ids: vec![],
            discard_pile_card_ids: vec![],
            exhaust_pile_card_ids: vec![],
            fights_won: 5,
            play_deck_size: 3,
            play_deck_card_ids: vec!["strike".into(), "defend".into(), "barbs".into()],
            collection_size: 5,
            collection_card_ids: vec![
                "strike".into(),
                "defend".into(),
                "barbs".into(),
                "heavy_strike".into(),
                "rally".into(),
            ],
            bonus_draw: 0,
            bonus_energy: 0,
            innate_card_ids: vec![],
            reward_cards: vec![],
            acquired_card: None,
            choice_cards: vec![],
            rebuild_slot_index: 0,
            rebuild_partial_deck_card_ids: vec![],
            rebuild_remaining_card_ids: vec![],
            pending_discards: 0,
            player_took_damage_last_turn: false,
            cards_played_this_turn: 0,
            play_deck_cards: vec![],
            subclass: String::new(),
            race: String::new(),
            background: String::new(),
        }
    }

    fn dummy_card_obs(card_id: &str) -> CardObs {
        CardObs {
            card_id: card_id.to_string(),
            name: card_id.to_string(),
            cost: 1,
            tags: vec![],
            description: String::new(),
            playable: false,
            total_damage: 6,
            total_block: 0,
            exhaust: false,
            draw_count: 0,
            energy_gain: 0,
            enemy_statuses: vec![],
            self_statuses: vec![],
            targets_all: false,
            hp_change: 0,
            innate: false,
            recycle: false,
            has_conditional: false,
            concentration: false,
        }
    }

    #[test]
    fn swap_into_deck_encodes_acquired_card_in_slot_a() {
        let content = decker_engine::content_tables::ContentTables::empty();
        let mut obs = dummy_obs();
        obs.acquired_card = Some(dummy_card_obs("heavy_strike"));

        let action = GauntletAction::SwapIntoDeck(0); // replace deck[0]
        let feats = action_features_single(&action, &obs, &content);

        assert_eq!(feats.len(), ACTION_FEAT_DIM);
        assert_eq!(feats[7], 1.0); // SwapIntoDeck one-hot

        // Slot A (acquired card) should have heavy_strike identity one-hot
        let slot_a_start = ACTION_TYPES; // 12
        let identity_off = slot_a_start + 52; // card identity starts at offset 52 within card feats
        let hs_idx = card_vocab_index("heavy_strike");
        assert_eq!(
            feats[identity_off + hs_idx],
            1.0,
            "acquired card identity not set in slot A"
        );

        // Slot B should NOT have the acquired card — it should have the deck card
        let slot_b_start = ACTION_TYPES + CARD_FEAT_DIM;
        let b_identity_off = slot_b_start + 52;
        // deck[0] = "strike", but we don't have content loaded, so it'll be zeros
        // (no def found). The important thing is slot A has the acquired card.
        assert_eq!(
            feats[b_identity_off + hs_idx],
            0.0,
            "acquired card should not be in slot B"
        );
    }

    #[test]
    fn overflow_remove_encodes_removed_collection_card() {
        let content = decker_engine::content_tables::ContentTables::empty();
        let mut obs = dummy_obs();
        obs.phase_type = ObservedPhase::CollectionOverflow;
        obs.choice_cards = vec![dummy_card_obs("rally"), dummy_card_obs("heavy_strike")];

        let action = GauntletAction::RemoveCollectionCard(1);
        let feats = action_features_single(&action, &obs, &content);

        assert_eq!(feats.len(), ACTION_FEAT_DIM);
        assert_eq!(feats[5], 1.0);

        // Slot B should have heavy_strike.
        let identity_off = ACTION_TYPES + CARD_FEAT_DIM + 52;
        let hs_idx = card_vocab_index("heavy_strike");
        assert_eq!(
            feats[identity_off + hs_idx],
            1.0,
            "overflow removal should encode the removed collection card in slot B"
        );
    }

    #[test]
    fn choose_deck_slot_card_encodes_assignment() {
        let content = decker_engine::content_tables::ContentTables::empty();
        let mut obs = dummy_obs();
        obs.phase_type = ObservedPhase::DeckRebuild;
        obs.choice_cards = vec![dummy_card_obs("rally"), dummy_card_obs("heavy_strike")];
        obs.rebuild_slot_index = 0;

        let action = GauntletAction::ChooseDeckSlotCard(1);
        let feats = action_features_single(&action, &obs, &content);

        assert_eq!(feats.len(), ACTION_FEAT_DIM);
        assert_eq!(feats[9], 1.0);

        // Slot A should have heavy_strike (assigned card)
        let a_identity_off = ACTION_TYPES + 52;
        let hs_idx = card_vocab_index("heavy_strike");
        assert_eq!(
            feats[a_identity_off + hs_idx],
            1.0,
            "ChooseDeckSlotCard should encode the assigned card in slot A"
        );
    }

    #[test]
    fn non_combat_histograms_populated() {
        let mut obs = dummy_obs();
        obs.phase_type = ObservedPhase::DeckRebuild;
        obs.choice_cards = vec![dummy_card_obs("rally"), dummy_card_obs("heavy_strike")];
        obs.rebuild_partial_deck_card_ids = vec!["strike".into(), "defend".into()];
        obs.rebuild_remaining_card_ids =
            vec!["barbs".into(), "heavy_strike".into(), "rally".into()];

        let feats = global_features(&obs);
        assert_eq!(feats.len(), GLOBAL_DIM);

        // Histogram region starts at offset 43, 6 slots of VOCAB_SIZE each.
        let hist_start = 43;

        // Slot 1: play deck histogram — should have nonzero entries
        let slot1 = &feats[hist_start..hist_start + VOCAB_SIZE];
        assert!(
            slot1.iter().any(|&x| x > 0.0),
            "play deck histogram should be non-zero"
        );

        // Slot 2: collection histogram — should have nonzero entries
        let slot2 = &feats[hist_start + VOCAB_SIZE..hist_start + VOCAB_SIZE * 2];
        assert!(
            slot2.iter().any(|&x| x > 0.0),
            "collection histogram should be non-zero"
        );

        // Slot 3: current choice-set histogram — should have nonzero entries
        let slot3 = &feats[hist_start + VOCAB_SIZE * 2..hist_start + VOCAB_SIZE * 3];
        assert!(
            slot3.iter().any(|&x| x > 0.0),
            "choice histogram should be non-zero"
        );

        // Slot 4: partial rebuild deck — should be nonzero during rebuild
        let slot4 = &feats[hist_start + VOCAB_SIZE * 3..hist_start + VOCAB_SIZE * 4];
        assert!(
            slot4.iter().any(|&x| x > 0.0),
            "partial rebuild histogram should be non-zero"
        );

        // Slot 5: innate cards — zero when there are none
        let slot5 = &feats[hist_start + VOCAB_SIZE * 4..hist_start + VOCAB_SIZE * 5];
        assert!(
            slot5.iter().all(|&x| x == 0.0),
            "innate histogram should be zero when there are no innate cards"
        );

        // Slot 6: remaining rebuild inventory — should be nonzero during rebuild
        let slot6 = &feats[hist_start + VOCAB_SIZE * 5..hist_start + VOCAB_SIZE * 6];
        assert!(
            slot6.iter().any(|&x| x > 0.0),
            "remaining inventory histogram should be non-zero"
        );
    }
}
