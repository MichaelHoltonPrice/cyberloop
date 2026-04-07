//! String-based card identifiers.
//!
//! `CardId` is a type alias for `String` so that `HashMap<CardId, V>.get("strike")`
//! works via the `Borrow` trait without needing `.into()` at every call site.

pub type CardId = String;

// ── Basics (Universal) ──────────────────────────────────────────────────
pub const STRIKE: &str = "strike";
pub const DEFEND: &str = "defend";

// ── Curse ───────────────────────────────────────────────────────────────
pub const WOUND: &str = "wound";

// ── Fighter class cards ─────────────────────────────────────────────────
pub const SECOND_WIND: &str = "second_wind";
pub const ENERGY_SURGE: &str = "energy_surge";
pub const EXTRA_ATTACK: &str = "extra_attack";
pub const RALLY: &str = "rally";

// ── Fighter themed basics ───────────────────────────────────────────────
pub const BARBS: &str = "barbs";
pub const HEAVY_STRIKE: &str = "heavy_strike";
pub const MEASURED_STRIKE: &str = "measured_strike";

// ── Defense subclass progression ────────────────────────────────────────
pub const IRON_GUARD: &str = "iron_guard";
pub const SHIELD_BASH: &str = "shield_bash";
pub const BRACE: &str = "brace";
pub const UNBREAKABLE: &str = "unbreakable";

// ── Two-Handed subclass progression ─────────────────────────────────────
pub const INTIMIDATING_BLOW: &str = "intimidating_blow";
pub const RECKLESS_ATTACK: &str = "reckless_attack";
pub const SUNDERING_BLOW: &str = "sundering_blow";
pub const REAVING_STRIKE: &str = "reaving_strike";

// ── Dueling subclass progression ────────────────────────────────────────
pub const SIZING_UP: &str = "sizing_up";
pub const PRECISE_CUT: &str = "precise_cut";
pub const EXPOSE_WEAKNESS: &str = "expose_weakness";
pub const CALCULATED_STRIKE: &str = "calculated_strike";
pub const READ_AND_REACT: &str = "read_and_react";
pub const FLURRY_OF_CUTS: &str = "flurry_of_cuts";
pub const DEFT_STRIKE: &str = "deft_strike";
pub const RIPOSTE: &str = "riposte";
pub const FEINT: &str = "feint";
pub const PATIENT_STRIKE: &str = "patient_strike";
pub const PERFECT_RHYTHM: &str = "perfect_rhythm";
pub const PERFECT_READ: &str = "perfect_read";

// ── Martial reward pool ────────────────────────────────────────────────
pub const POWER_STRIKE: &str = "power_strike";
pub const QUICK_SLASH: &str = "quick_slash";
pub const VICIOUS_STRIKE: &str = "vicious_strike";
pub const DOUBLE_STRIKE: &str = "double_strike";
pub const BRACE_FOR_IMPACT: &str = "brace_for_impact";
pub const HOLD_THE_LINE: &str = "hold_the_line";
pub const TAUNT: &str = "taunt";
pub const FOCUS: &str = "focus";
pub const MARTIAL_ASCENDANCY: &str = "martial_ascendancy";

// ── Legendary capstones ────────────────────────────────────────────────
pub const IRON_FORTRESS: &str = "iron_fortress";
pub const TITANS_FURY: &str = "titans_fury";
pub const COUP_DE_GRACE: &str = "coup_de_grace";

// ── New dueling progression cards ─────────────────────────────────────
pub const ADRENALINE_RUSH: &str = "adrenaline_rush";
pub const EXPLOIT_OPENING: &str = "exploit_opening";
pub const MOMENTUM: &str = "momentum";

// ── Race cards ────────────────────────────────────────────────────────
pub const IMPROVISE: &str = "improvise";
pub const FAE_ANCESTRY: &str = "fae_ancestry";
pub const BLOOD_PRICE: &str = "blood_price";
pub const STONEWALL: &str = "stonewall";
pub const FLASH_BANG: &str = "flash_bang";
pub const NIMBLE_DODGE: &str = "nimble_dodge";
pub const SAVAGE_CHARGE: &str = "savage_charge";
pub const SHIV: &str = "shiv";
pub const DRAGON_BREATH: &str = "dragon_breath";
pub const POUNCE: &str = "pounce";

// ── Background cards ────────────────────────────────────────────────────
pub const BATTLE_DISCIPLINE: &str = "battle_discipline";
pub const STUDIED_ANALYSIS: &str = "studied_analysis";
pub const MINOR_BLESSING: &str = "minor_blessing";
pub const DIRTY_TRICK: &str = "dirty_trick";
pub const COMMANDING_PRESENCE: &str = "commanding_presence";
pub const SURVIVAL_INSTINCT: &str = "survival_instinct";
pub const IMPROVISED_WEAPON: &str = "improvised_weapon";
pub const SEA_LEGS: &str = "sea_legs";
pub const BACKSTAB: &str = "backstab";
pub const INNER_FOCUS: &str = "inner_focus";
pub const RESOURCEFUL: &str = "resourceful";
pub const DAZZLE: &str = "dazzle";

// ── Defense subclass: Bulwark synergy group ─────────────────────────────
pub const FORTIFY: &str = "fortify";
pub const ENTRENCH: &str = "entrench";
pub const STALWART_DEFENSE: &str = "stalwart_defense";
pub const SHIELD_WALL: &str = "shield_wall";
pub const AEGIS_ETERNAL: &str = "aegis_eternal";

// ── Defense subclass: Reprisal synergy group ────────────────────────────
pub const SPIKED_ARMOR: &str = "spiked_armor";
pub const RETRIBUTION: &str = "retribution";
pub const THORNED_CARAPACE: &str = "thorned_carapace";
pub const BARBED_BULWARK: &str = "barbed_bulwark";
pub const WRATH_OF_THORNS: &str = "wrath_of_thorns";

// ── Two-Handed subclass: Crusher synergy group ──────────────────────────
pub const BRUTAL_SWING: &str = "brutal_swing";
pub const SAVAGE_PRESENCE: &str = "savage_presence";
pub const DEMOLISH: &str = "demolish";
pub const WRATH_OF_THE_GIANT: &str = "wrath_of_the_giant";
pub const EXECUTIONERS_BLOW: &str = "executioners_blow";

// ── Two-Handed subclass: Berserker synergy group ────────────────────────
pub const BLOOD_FRENZY: &str = "blood_frenzy";
pub const RAGING_BLOW: &str = "raging_blow";
pub const BERSERK_ROAR: &str = "berserk_roar";
pub const UNLEASH_FURY: &str = "unleash_fury";
pub const DEATHWISH: &str = "deathwish";

// ── Barbarian class card ────────────────────────────────────────────────
pub const BLOODLUST: &str = "bloodlust";

// ── Barbarian themed basics ─────────────────────────────────────────────
pub const RAGING_STRIKE: &str = "raging_strike";
pub const TOTEM_GUARD: &str = "totem_guard";
pub const FRENZIED_SLASH: &str = "frenzied_slash";

// ── Barbarian shared auto-grants ────────────────────────────────────────
pub const BATTLE_FURY: &str = "battle_fury";
pub const THICK_SKIN: &str = "thick_skin";
pub const PRIMAL_SURGE: &str = "primal_surge";
pub const UNDYING_RAGE: &str = "undying_rage";

// ── Berserker: Bloodrage synergy group ──────────────────────────────────
pub const BLOOD_OFFERING: &str = "blood_offering";
pub const FURY_UNLEASHED: &str = "fury_unleashed";
pub const BLOODBATH: &str = "bloodbath";
pub const PAIN_IS_POWER: &str = "pain_is_power";
pub const BERSERKER_DEATHWISH: &str = "berserker_deathwish";

// ── Berserker: Rampage synergy group ────────────────────────────────────
pub const WILD_SWING: &str = "wild_swing";
pub const BERSERK_FLURRY: &str = "berserk_flurry";
pub const SAVAGE_MOMENTUM: &str = "savage_momentum";
pub const UNSTOPPABLE: &str = "unstoppable";
pub const RAMPAGE: &str = "rampage";

// ── Totem Warrior: Spirit Shield synergy group ──────────────────────────
pub const SPIRIT_WARD: &str = "spirit_ward";
pub const ANCESTRAL_SHIELD: &str = "ancestral_shield";
pub const WARDING_TOTEM_CARD: &str = "warding_totem_card";
pub const FORTIFIED_RAGE: &str = "fortified_rage";
pub const UNBREAKING_SPIRIT: &str = "unbreaking_spirit";

// ── Totem Warrior: Ancestral synergy group ──────────────────────────────
pub const WAR_CRY: &str = "war_cry";
pub const SPIRIT_MEND: &str = "spirit_mend";
pub const VENGEFUL_ANCESTORS: &str = "vengeful_ancestors";
pub const TOTEM_OF_RENEWAL: &str = "totem_of_renewal";
pub const ANCESTORS_EMBRACE: &str = "ancestors_embrace";

// ── Frenzy: Adrenaline synergy group ────────────────────────────────────
pub const DESPERATE_STRIKE: &str = "desperate_strike";
pub const ADRENALINE_SPIKE: &str = "adrenaline_spike";
pub const FERAL_INSTINCT: &str = "feral_instinct";
pub const BERSERKERS_TRANCE_CARD: &str = "berserkers_trance_card";
pub const DEATHS_DOOR: &str = "deaths_door";

// ── Frenzy: Overwhelm synergy group ─────────────────────────────────────
pub const FLAILING_STRIKE: &str = "flailing_strike";
pub const FRENZY_CARD: &str = "frenzy_card";
pub const WHIRLWIND_FURY: &str = "whirlwind_fury";
pub const ONE_THOUSAND_CUTS: &str = "one_thousand_cuts";

// ── Monk class card ─────────────────────────────────────────────────────
pub const INNER_PEACE: &str = "inner_peace";

// ── Monk shared pool (level 1) ──────────────────────────────────────────
pub const KI_FOCUS: &str = "ki_focus";
pub const KI_STRIKE: &str = "ki_strike";
pub const KI_GUARD: &str = "ki_guard";

// ── Monk shared auto-grants ─────────────────────────────────────────────
pub const CENTERED_STRIKE: &str = "centered_strike";
pub const SWIFT_FOOTWORK: &str = "swift_footwork";
pub const KI_SURGE: &str = "ki_surge";
pub const MEDITATION: &str = "meditation";

// ── Open Hand: Flurry synergy ───────────────────────────────────────────
pub const RAPID_STRIKES: &str = "rapid_strikes";
pub const PALM_STRIKE: &str = "palm_strike";
pub const WHIRLWIND_KICK: &str = "whirlwind_kick";
pub const HUNDRED_FISTS: &str = "hundred_fists";
pub const ENDLESS_BARRAGE: &str = "endless_barrage";

// ── Open Hand: Ki Burst synergy ─────────────────────────────────────────
pub const FOCUSED_STRIKE: &str = "focused_strike";
pub const KI_SHIELD: &str = "ki_shield";
pub const KI_CHANNELING: &str = "ki_channeling";
pub const QUIVERING_PALM: &str = "quivering_palm";

// ── Open Hand: Ultimate ─────────────────────────────────────────────────
pub const TRANSCENDENCE: &str = "transcendence";

// ── Way of Shadow: Pressure Point synergy ───────────────────────────────
pub const NERVE_STRIKE: &str = "nerve_strike";
pub const PRESSURE_POINT_STRIKE: &str = "pressure_point_strike";
pub const DIM_MAK: &str = "dim_mak";
pub const CRIPPLING_BLOW: &str = "crippling_blow";
pub const WEAKENING_AURA: &str = "weakening_aura";
pub const DEATH_TOUCH: &str = "death_touch";

// ── Way of Shadow: Evasion synergy ──────────────────────────────────────
pub const SHADOW_STEP: &str = "shadow_step";
pub const DEFLECTING_PALM: &str = "deflecting_palm";
pub const COUNTER_STANCE: &str = "counter_stance";
pub const PHANTOM_FORM: &str = "phantom_form";

// ── Way of Shadow: Ultimate ─────────────────────────────────────────────
pub const SHADOW_KILL: &str = "shadow_kill";

// ── Iron Fist: Stance Flow synergy ──────────────────────────────────────
pub const STANCE_SHIFT: &str = "stance_shift";
pub const TIGERS_FURY: &str = "tigers_fury";
pub const CRANES_WING: &str = "cranes_wing";
pub const FLOWING_WATER: &str = "flowing_water";
pub const MONK_DRAGONS_BREATH: &str = "monk_dragons_breath";
pub const PERFECT_HARMONY: &str = "perfect_harmony";

// ── Iron Fist: Iron Will synergy ────────────────────────────────────────
pub const IRON_SKIN: &str = "iron_skin";
pub const KI_BARRIER: &str = "ki_barrier";
pub const STONE_BODY: &str = "stone_body";
pub const DIAMOND_SOUL: &str = "diamond_soul";

// ── Iron Fist: Ultimate ─────────────────────────────────────────────────
pub const FIST_OF_NORTH_STAR: &str = "fist_of_north_star";

// ── Warlock class cards ─────────────────────────────────────────────────
pub const ELDRITCH_BLAST: &str = "eldritch_blast";
pub const HEX: &str = "hex";

// ── Warlock shared cards ────────────────────────────────────────────────
pub const DARK_BARGAIN: &str = "dark_bargain";
pub const MALEDICT: &str = "maledict";
pub const PACT_BARRIER: &str = "pact_barrier";
pub const SIPHON_BOLT: &str = "siphon_bolt";

// ── Warlock Infernal: Infernal Burn synergy ─────────────────────────────
pub const HELLBRAND: &str = "hellbrand";
pub const CINDER_FEAST: &str = "cinder_feast";
pub const BALEFIRE_CATACLYSM: &str = "balefire_cataclysm";

// ── Warlock Infernal: Infernal Sustain synergy ──────────────────────────
pub const BLOOD_TITHE: &str = "blood_tithe";
pub const FEAST_OF_CINDERS: &str = "feast_of_cinders";

// ── Warlock Hexblade: Hexblade Duel synergy ─────────────────────────────
pub const PACT_BLADE: &str = "pact_blade";
pub const SOUL_PARRY: &str = "soul_parry";
pub const DUSK_DUEL: &str = "dusk_duel";

// ── Warlock Hexblade: Hexblade Curse synergy ────────────────────────────
pub const BRAND_THE_WEAK: &str = "brand_the_weak";
pub const BLACK_OATH: &str = "black_oath";

// ── Warlock Void: Void Debuff synergy ───────────────────────────────────
pub const VOID_GAZE: &str = "void_gaze";
pub const STARLESS_WHISPER: &str = "starless_whisper";
pub const ENTROPY_FIELD: &str = "entropy_field";

// ── Warlock Void: Void Annihilation synergy ─────────────────────────────
pub const OBLIVION_TIDE: &str = "oblivion_tide";
pub const COSMIC_EXTINCTION: &str = "cosmic_extinction";

// ── Paladin shared cards ───────────────────────────────────────────────
pub const HOLY_STRIKE: &str = "holy_strike";
pub const LAY_ON_HANDS: &str = "lay_on_hands";
pub const PRAYER_OF_VALOR: &str = "prayer_of_valor";
pub const CONSECRATION: &str = "consecration";
pub const DIVINE_BULWARK: &str = "divine_bulwark";

// ── Paladin: Devotion style ────────────────────────────────────────────
pub const WARDING_SLASH: &str = "warding_slash";
pub const SHIELD_OF_FAITH: &str = "shield_of_faith";
pub const BASTION_DRIVE: &str = "bastion_drive";
pub const HOLY_BASTION: &str = "holy_bastion";

// ── Paladin: Vengeance style ───────────────────────────────────────────
pub const AVENGING_STRIKE: &str = "avenging_strike";
pub const BLADE_OF_WRATH: &str = "blade_of_wrath";
pub const DIVINE_JUDGMENT: &str = "divine_judgment";
pub const ZEALOUS_RUSH: &str = "zealous_rush";

// ── Paladin: Radiance style ───────────────────────────────────────────
pub const RADIANT_BURST: &str = "radiant_burst";
pub const BEACON_OF_LIGHT: &str = "beacon_of_light";
pub const SOLAR_FLARE: &str = "solar_flare";
pub const DAWNS_BLESSING: &str = "dawns_blessing";

// ── Paladin: Capstone ──────────────────────────────────────────────────
pub const AVATAR_OF_RADIANCE: &str = "avatar_of_radiance";

// ── Rogue class card ──────────────────────────────────────────────────
pub const TRICKS_OF_THE_TRADE: &str = "tricks_of_the_trade";

// ── Rogue shared auto-grants ─────────────────────────────────────────
pub const SNEAK_ATTACK: &str = "sneak_attack";
pub const CUNNING_ACTION: &str = "cunning_action";
pub const ROGUE_EVASION: &str = "rogue_evasion";
pub const PREPARATION: &str = "preparation";

// ── Assassin synergy: assassination ──────────────────────────────────
pub const ASSASSINATE: &str = "assassinate";
pub const ROGUE_SHADOW_STEP: &str = "rogue_shadow_step";
pub const DEATH_MARK: &str = "death_mark";
pub const KILLING_BLOW: &str = "killing_blow";

// ── Pirate synergy: pirate_blade / pirate_trick ─────────────────────
pub const CUTLASS_FLURRY: &str = "cutlass_flurry";
pub const GRAPPLING_HOOK: &str = "grappling_hook";
pub const DIRTY_FIGHTING: &str = "dirty_fighting";
pub const BROADSIDE: &str = "broadside";

// ── Trickster synergy: trickster_smoke / trickster_finesse ──────────
pub const SMOKE_BOMB: &str = "smoke_bomb";
pub const MISDIRECTION: &str = "misdirection";
pub const FAN_OF_KNIVES: &str = "fan_of_knives";
pub const ACE_IN_THE_HOLE: &str = "ace_in_the_hole";

// ── Druid class cards ──────────────────────────────────────────────────
pub const WILD_SHAPE_BEAR: &str = "wild_shape_bear";
pub const WILD_SHAPE_EAGLE: &str = "wild_shape_eagle";
pub const WILD_SHAPE_WOLF: &str = "wild_shape_wolf";
pub const NATURES_WRATH: &str = "natures_wrath";

// ── Druid shared pool / auto-grants ────────────────────────────────────
pub const BARK_SKIN: &str = "bark_skin";
pub const REJUVENATION: &str = "rejuvenation";
pub const MOONBEAM: &str = "moonbeam";
pub const ENTANGLE: &str = "entangle";

// ── Bear synergy ───────────────────────────────────────────────────────
pub const BEAR_MAUL: &str = "bear_maul";
pub const THICK_HIDE: &str = "thick_hide";
pub const URSINE_CHARGE: &str = "ursine_charge";
pub const PRIMAL_ROAR: &str = "primal_roar";

// ── Eagle synergy ──────────────────────────────────────────────────────
pub const EAGLE_DIVE: &str = "eagle_dive";
pub const SWOOPING_STRIKE: &str = "swooping_strike";
pub const WIND_RIDER: &str = "wind_rider";
pub const TEMPEST_TALONS: &str = "tempest_talons";

// ── Wolf synergy ───────────────────────────────────────────────────────
pub const PACK_TACTICS: &str = "pack_tactics";
pub const HOWL: &str = "howl";
pub const COORDINATED_STRIKE: &str = "coordinated_strike";
pub const ALPHAS_COMMAND: &str = "alphas_command";

// ── Druid capstone ─────────────────────────────────────────────────────
pub const ARCHDRUID: &str = "archdruid";

// ── Wizard shared cards ────────────────────────────────────────────────
pub const FIRE_BOLT: &str = "fire_bolt";
pub const MAGE_ARMOR: &str = "mage_armor";
pub const ARCANE_INTELLECT: &str = "arcane_intellect";
pub const SHIELD_SPELL: &str = "shield_spell";

// ── Wizard themed basics (subclass starters) ──────────────────────────
pub const EMBER_BOLT: &str = "ember_bolt";
pub const ARCANE_WARD: &str = "arcane_ward";
pub const PRESCIENCE: &str = "prescience";

// ── Evocation subclass progression ────────────────────────────────────
pub const FIREBALL: &str = "fireball";
pub const SCORCHING_RAY: &str = "scorching_ray";
pub const LIGHTNING_BOLT: &str = "lightning_bolt";
pub const FLAME_SHIELD: &str = "flame_shield";

// ── Abjuration subclass progression ───────────────────────────────────
pub const COUNTERSPELL: &str = "counterspell";
pub const WARD: &str = "ward";
pub const DISPEL: &str = "dispel";
pub const GLOBE_OF_PROTECTION: &str = "globe_of_protection";

// ── Divination subclass progression ───────────────────────────────────
pub const FORESIGHT: &str = "foresight";
pub const SCRYING: &str = "scrying";
pub const PORTENT: &str = "portent";
pub const TIME_STOP: &str = "time_stop";

// ── Wizard capstone ───────────────────────────────────────────────────
pub const WISH: &str = "wish";

// ── Sorcerer starter cards: Wild Mage ─────────────────────────────────
pub const SPARK_BOLT: &str = "spark_bolt";
pub const CHANNEL: &str = "channel";
pub const SURGE_WAVE: &str = "surge_wave";
pub const ARCANE_SHIELD: &str = "arcane_shield";
pub const WILD_SURGE: &str = "wild_surge";

// ── Sorcerer starter cards: Draconic Bloodline ────────────────────────
pub const CLAW_STRIKE: &str = "claw_strike";
pub const SCALE_WARD: &str = "scale_ward";
pub const DRACONIC_FOCUS: &str = "draconic_focus";
pub const DRAGONS_RESILIENCE: &str = "dragons_resilience";
pub const DRACONIC_BOLT: &str = "draconic_bolt";

// ── Sorcerer starter cards: Shadow Sorcerer ───────────────────────────
pub const SHADOW_BOLT: &str = "shadow_bolt";
pub const DARK_SHROUD: &str = "dark_shroud";
pub const SIPHON: &str = "siphon";
pub const NIGHT_VEIL: &str = "night_veil";
pub const UMBRAL_STRIKE: &str = "umbral_strike";

// ── Sorcerer shared pool cards ─────────────────────────────────────────
pub const ARCANE_PULSE: &str = "arcane_pulse";
pub const MANA_INFUSION: &str = "mana_infusion";
pub const SPELL_SURGE: &str = "spell_surge";
pub const OVERLOAD: &str = "overload";

// ── Wild Mage pool: Surge synergy ─────────────────────────────────────
pub const VOLATILE_BURST: &str = "volatile_burst";
pub const CHAIN_LIGHTNING: &str = "chain_lightning";
pub const SURGE_NOVA: &str = "surge_nova";
pub const WILD_DISCHARGE: &str = "wild_discharge";
pub const SORCERER_CATACLYSM: &str = "sorcerer_cataclysm";

// ── Wild Mage pool: Chaos synergy ─────────────────────────────────────
pub const CHAOS_BOLT: &str = "chaos_bolt";
pub const UNRAVELING_HEX: &str = "unraveling_hex";
pub const ENTROPY_CASCADE: &str = "entropy_cascade";
pub const SORCERER_PANDEMONIUM: &str = "sorcerer_pandemonium";

// ── Draconic pool: Heritage synergy ───────────────────────────────────
pub const DRAGON_SCALE: &str = "dragon_scale";
pub const ANCESTRAL_RESILIENCE: &str = "ancestral_resilience";
pub const SCALED_REBUKE: &str = "scaled_rebuke";
pub const PRIMORDIAL_RESILIENCE: &str = "primordial_resilience";

// ── Draconic pool: Wrath synergy ──────────────────────────────────────
pub const SORCERER_DRAGONS_BREATH: &str = "sorcerer_dragons_breath";
pub const DRACONIC_SURGE: &str = "draconic_surge";
pub const ANCIENT_POWER: &str = "ancient_power";
pub const DRAGONFIRE_ASCENSION: &str = "dragonfire_ascension";

// ── Shadow Sorcerer pool: Drain synergy ───────────────────────────────
pub const SHADOW_TAP: &str = "shadow_tap";
pub const NECROTIC_SIPHON: &str = "necrotic_siphon";
pub const LIFE_DRAIN: &str = "life_drain";
pub const VOID_FEAST: &str = "void_feast";
pub const ETERNAL_HUNGER: &str = "eternal_hunger";

// ── Shadow Sorcerer pool: Terrors synergy ─────────────────────────────
pub const DREAD_BOLT: &str = "dread_bolt";
pub const TERRORIZE: &str = "terrorize";
pub const SHADOW_EXPLOIT: &str = "shadow_exploit";
pub const FINAL_NIGHTMARE: &str = "final_nightmare";

// ── Sorcerer ultimate ─────────────────────────────────────────────────
pub const SORCERERS_APOTHEOSIS: &str = "sorcerers_apotheosis";
