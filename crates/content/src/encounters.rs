//! Encounter generation for the Gauntlet mode.
//!
//! Encounters resolve to plain enemy ID lists for combat, organized with
//! lightweight metadata for grouping by player level, theme, and background.

use decker_engine::ids::EnemyId;
use decker_engine::rng::GameRng;

use crate::enemies;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterTheme {
    Goblins,
    Undead,
    Vermin,
    Slimes,
    Beasts,
    Bandits,
    Constructs,
    Cult,
    Fiends,
    Fungal,
    Mixed,
}

impl EncounterTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Goblins => "goblins",
            Self::Undead => "undead",
            Self::Vermin => "vermin",
            Self::Slimes => "slimes",
            Self::Beasts => "beasts",
            Self::Bandits => "bandits",
            Self::Constructs => "constructs",
            Self::Cult => "cult",
            Self::Fiends => "fiends",
            Self::Fungal => "fungal",
            Self::Mixed => "mixed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterBackground {
    RoadAmbush,
    GoblinCamp,
    InfestedDen,
    SlimeCave,
    CaveNest,
    RuinedOutpost,
    Graveyard,
    Crypt,
    AncientVault,
    ForestClearing,
    BanditCamp,
    CultSanctum,
    TrollDen,
    DrakeLair,
    FiendGate,
    UndeadCrypt,
    MercenaryFort,
    DragonApproach,
}

impl EncounterBackground {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RoadAmbush => "road_ambush",
            Self::GoblinCamp => "goblin_camp",
            Self::InfestedDen => "infested_den",
            Self::SlimeCave => "slime_cave",
            Self::CaveNest => "cave_nest",
            Self::RuinedOutpost => "ruined_outpost",
            Self::Graveyard => "graveyard",
            Self::Crypt => "crypt",
            Self::AncientVault => "ancient_vault",
            Self::ForestClearing => "forest_clearing",
            Self::BanditCamp => "bandit_camp",
            Self::CultSanctum => "cult_sanctum",
            Self::TrollDen => "troll_den",
            Self::DrakeLair => "drake_lair",
            Self::FiendGate => "fiend_gate",
            Self::UndeadCrypt => "undead_crypt",
            Self::MercenaryFort => "mercenary_fort",
            Self::DragonApproach => "dragon_approach",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncounterTemplate {
    pub key: &'static str,
    pub player_level: u32,
    pub theme: EncounterTheme,
    pub background: EncounterBackground,
    pub enemy_ids: &'static [&'static str],
}

impl EncounterTemplate {
    fn to_enemy_ids(self) -> Vec<EnemyId> {
        self.enemy_ids.iter().map(|id| (*id).into()).collect()
    }

    fn hash_input(self) -> String {
        format!(
            "{}|lvl{}|{}|{}|{}",
            self.key,
            self.player_level,
            self.theme.as_str(),
            self.background.as_str(),
            self.enemy_ids.join(",")
        )
    }
}

// ══════════════════════════════════════════════════════════════════════════
// Level 1 — The Threshold (Fights 0-3)
// Fewer enemies, higher individual threat. Even good play loses some HP.
// Enemies that buff Empowered create "kill now or face escalating damage."
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_1_ENCOUNTERS: &[EncounterTemplate] = &[
    // Goblin Patrol: paired pressure, 15-17 damage/turn
    EncounterTemplate {
        key: "goblin_patrol_a",
        player_level: 1,
        theme: EncounterTheme::Goblins,
        background: EncounterBackground::RoadAmbush,
        enemy_ids: &["goblin_grunt"],
    },
    EncounterTemplate {
        key: "goblin_patrol_b",
        player_level: 1,
        theme: EncounterTheme::Goblins,
        background: EncounterBackground::GoblinCamp,
        enemy_ids: &["goblin_grunt", "goblin_archer"],
    },
    // Vermin Nest: swarm of squishy but hard-hitting rats
    EncounterTemplate {
        key: "vermin_nest",
        player_level: 1,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["giant_rat", "giant_rat", "giant_rat"],
    },
    // Cave Fauna: solo scavenger — solid 1v1
    EncounterTemplate {
        key: "cave_fauna",
        player_level: 1,
        theme: EncounterTheme::Slimes,
        background: EncounterBackground::SlimeCave,
        enemy_ids: &["scavenger"],
    },
    // Bandit Scouts: mixed targets, priority decision
    EncounterTemplate {
        key: "bandit_scouts_a",
        player_level: 1,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::RoadAmbush,
        enemy_ids: &["scavenger", "giant_rat"],
    },
    // Bandit Thug: solo mini-boss, strength ramp — hardest level 1 fight
    EncounterTemplate {
        key: "bandit_scouts_b",
        player_level: 1,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::BanditCamp,
        enemy_ids: &["bandit_thug"],
    },
    // Restless Dead: paired skeletons with strength ramp
    EncounterTemplate {
        key: "restless_dead",
        player_level: 1,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Graveyard,
        enemy_ids: &["skeleton", "skeleton"],
    },
    // Fungal Growth: debuff + chaff, teaches "kill the debuffer first"
    EncounterTemplate {
        key: "fungal_growth",
        player_level: 1,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::InfestedDen,
        enemy_ids: &["sporecap", "giant_rat"],
    },
    // Blood Pests: DOT pressure, teaches tempo
    EncounterTemplate {
        key: "blood_pests",
        player_level: 1,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["blood_moth", "blood_moth"],
    },
    // Animated Junk: defensive enemy, teaches sustained damage
    EncounterTemplate {
        key: "animated_junk",
        player_level: 1,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::RuinedOutpost,
        enemy_ids: &["animated_shield"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 2 — Into the Dark (Fights 4-7)
// Budget 3-5. Light enemy synergies. One support enemy buffing others.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_2_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "goblin_war_party",
        player_level: 2,
        theme: EncounterTheme::Goblins,
        background: EncounterBackground::GoblinCamp,
        enemy_ids: &["goblin_shaman", "goblin_grunt"],
    },
    EncounterTemplate {
        key: "goblin_ranged_line",
        player_level: 2,
        theme: EncounterTheme::Goblins,
        background: EncounterBackground::RoadAmbush,
        enemy_ids: &["goblin_archer", "goblin_grunt"],
    },
    EncounterTemplate {
        key: "undead_patrol",
        player_level: 2,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Graveyard,
        enemy_ids: &["skeleton", "skeleton"],
    },
    EncounterTemplate {
        key: "zombie_wall",
        player_level: 2,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["zombie", "zombie"],
    },
    EncounterTemplate {
        key: "bandit_camp",
        player_level: 2,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::BanditCamp,
        enemy_ids: &["bandit_thug", "scavenger"],
    },
    EncounterTemplate {
        key: "bandit_ranged",
        player_level: 2,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::RoadAmbush,
        enemy_ids: &["bandit_archer", "bandit_thug"],
    },
    EncounterTemplate {
        key: "vermin_swarm",
        player_level: 2,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["giant_rat", "giant_rat", "giant_rat", "sporecap"],
    },
    EncounterTemplate {
        key: "plague_rats",
        player_level: 2,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::InfestedDen,
        enemy_ids: &["plague_rat", "plague_rat"],
    },
    EncounterTemplate {
        key: "insect_burrow",
        player_level: 2,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["cave_spider", "fire_beetle", "fire_beetle"],
    },
    EncounterTemplate {
        key: "tunnel_scavengers",
        player_level: 2,
        theme: EncounterTheme::Mixed,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["scavenger", "skeleton", "giant_rat"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 3 — Deepening Threats (Fights 8-11)
// Budget 4-6. Real synergies. Buff/debuff combos. Cult/undead develop.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_3_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "goblin_raiding_party",
        player_level: 3,
        theme: EncounterTheme::Goblins,
        background: EncounterBackground::GoblinCamp,
        enemy_ids: &["goblin_shaman", "goblin_grunt"],
    },
    EncounterTemplate {
        key: "bone_crypt",
        player_level: 3,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["bone_sentinel", "skeleton"],
    },
    EncounterTemplate {
        key: "undead_horde",
        player_level: 3,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Graveyard,
        enemy_ids: &["zombie", "zombie"],
    },
    EncounterTemplate {
        key: "cult_initiation",
        player_level: 3,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["cult_acolyte", "scavenger"],
    },
    EncounterTemplate {
        key: "beast_pack",
        player_level: 3,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::ForestClearing,
        enemy_ids: &["wolf", "wolf"],
    },
    EncounterTemplate {
        key: "boar_charge",
        player_level: 3,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::ForestClearing,
        enemy_ids: &["boar", "wolf"],
    },
    EncounterTemplate {
        key: "corrupted_growth",
        player_level: 3,
        theme: EncounterTheme::Fungal,
        background: EncounterBackground::InfestedDen,
        enemy_ids: &["blightcap", "sporecap"],
    },
    EncounterTemplate {
        key: "haunted_passage",
        player_level: 3,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["shadow_wisp", "shadow_wisp"],
    },
    EncounterTemplate {
        key: "mercenary_outriders",
        player_level: 3,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::RoadAmbush,
        enemy_ids: &["sellsword", "bandit_archer"],
    },
    EncounterTemplate {
        key: "sellsword_duel",
        player_level: 3,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::BanditCamp,
        enemy_ids: &["sellsword", "sellsword"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 4 — The Turning Point (Fights 12-15)
// Tough-tier enemies enter. Kill order matters. Cult/undead as factions.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_4_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "cultist_cell",
        player_level: 4,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["cult_channeler", "cult_acolyte"],
    },
    EncounterTemplate {
        key: "risen_horde",
        player_level: 4,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["bone_sentinel", "skeleton"],
    },
    EncounterTemplate {
        key: "troll_den",
        player_level: 4,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::TrollDen,
        enemy_ids: &["troll"],
    },
    EncounterTemplate {
        key: "construct_vault",
        player_level: 4,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::AncientVault,
        enemy_ids: &["stone_golem"],
    },
    EncounterTemplate {
        key: "construct_pair",
        player_level: 4,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::AncientVault,
        enemy_ids: &["animated_armor", "animated_shield"],
    },
    EncounterTemplate {
        key: "harpy_roost",
        player_level: 4,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::ForestClearing,
        enemy_ids: &["harpy"],
    },
    EncounterTemplate {
        key: "ambush_predator",
        player_level: 4,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["cave_stalker"],
    },
    EncounterTemplate {
        key: "deep_vermin",
        player_level: 4,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::InfestedDen,
        enemy_ids: &["rust_crawler", "fire_beetle"],
    },
    EncounterTemplate {
        key: "wight_ambush",
        player_level: 4,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Graveyard,
        enemy_ids: &["wight", "skeleton"],
    },
    EncounterTemplate {
        key: "cult_beasts",
        player_level: 4,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["cult_channeler", "wolf"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 5-6 — Faction Wars (Fights 16-23)
// Recognizable factions. Two-enemy synergies common.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_5_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "cult_ritual",
        player_level: 5,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["cult_channeler", "cult_acolyte"],
    },
    EncounterTemplate {
        key: "cult_invokers",
        player_level: 5,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["cult_invoker", "cult_acolyte"],
    },
    EncounterTemplate {
        key: "necromancer_servants",
        player_level: 5,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::UndeadCrypt,
        enemy_ids: &["bone_commander", "skeleton"],
    },
    EncounterTemplate {
        key: "undead_squad",
        player_level: 5,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["bone_commander", "wight"],
    },
    EncounterTemplate {
        key: "mercenary_company",
        player_level: 5,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::MercenaryFort,
        enemy_ids: &["sellsword_captain", "sellsword"],
    },
    EncounterTemplate {
        key: "specter_haunt",
        player_level: 5,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["specter", "shadow_wisp"],
    },
    EncounterTemplate {
        key: "ogre_territory",
        player_level: 5,
        theme: EncounterTheme::Goblins,
        background: EncounterBackground::RoadAmbush,
        enemy_ids: &["ogre_thug", "goblin_shaman"],
    },
    EncounterTemplate {
        key: "infernal_scouts",
        player_level: 5,
        theme: EncounterTheme::Fiends,
        background: EncounterBackground::FiendGate,
        enemy_ids: &["imp", "imp"],
    },
    EncounterTemplate {
        key: "corrupted_sanctum",
        player_level: 5,
        theme: EncounterTheme::Fungal,
        background: EncounterBackground::InfestedDen,
        enemy_ids: &["husk_whisperer", "sporecap"],
    },
    EncounterTemplate {
        key: "drake_nest",
        player_level: 5,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::DrakeLair,
        enemy_ids: &["young_drake", "young_drake"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 7-8 — Escalation (Fights 24-31)
// Elite-tier enemies appear. Dangerous anchors with support.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_7_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "cult_inner_circle",
        player_level: 7,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["cult_invoker", "cult_channeler"],
    },
    EncounterTemplate {
        key: "dark_priesthood",
        player_level: 7,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["dark_priest", "cult_acolyte"],
    },
    EncounterTemplate {
        key: "necromancer_vanguard",
        player_level: 7,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::UndeadCrypt,
        enemy_ids: &["corpse_knight", "bone_sentinel"],
    },
    EncounterTemplate {
        key: "double_commanders",
        player_level: 7,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["bone_commander", "wight"],
    },
    EncounterTemplate {
        key: "troll_warren",
        player_level: 7,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::TrollDen,
        enemy_ids: &["troll", "troll"],
    },
    EncounterTemplate {
        key: "golem_foundry",
        player_level: 7,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::AncientVault,
        enemy_ids: &["iron_golem"],
    },
    EncounterTemplate {
        key: "construct_trio",
        player_level: 7,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::AncientVault,
        enemy_ids: &["stone_golem", "animated_armor"],
    },
    EncounterTemplate {
        key: "wyvern_hunt",
        player_level: 7,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::ForestClearing,
        enemy_ids: &["wyvern"],
    },
    EncounterTemplate {
        key: "flamewrath_solo",
        player_level: 7,
        theme: EncounterTheme::Fiends,
        background: EncounterBackground::FiendGate,
        enemy_ids: &["flamewrath"],
    },
    EncounterTemplate {
        key: "predator_ambush",
        player_level: 7,
        theme: EncounterTheme::Vermin,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["cave_stalker", "cave_stalker"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 9-10 — The Gauntlet Tightens (Fights 32-39)
// Every encounter is a serious threat. Elites are common.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_9_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "necromancer_a",
        player_level: 9,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::UndeadCrypt,
        enemy_ids: &["necromancer", "bone_sentinel"],
    },
    EncounterTemplate {
        key: "necromancer_b",
        player_level: 9,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["necromancer", "corpse_knight"],
    },
    EncounterTemplate {
        key: "cult_sanctum",
        player_level: 9,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["high_cultist", "dark_priest"],
    },
    EncounterTemplate {
        key: "fiend_gate",
        player_level: 9,
        theme: EncounterTheme::Fiends,
        background: EncounterBackground::FiendGate,
        enemy_ids: &["flamewrath", "imp"],
    },
    EncounterTemplate {
        key: "iron_legion",
        player_level: 9,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::AncientVault,
        enemy_ids: &["iron_golem", "stone_golem"],
    },
    EncounterTemplate {
        key: "dragon_scouts",
        player_level: 9,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::DrakeLair,
        enemy_ids: &["wyvern", "wyvern"],
    },
    EncounterTemplate {
        key: "risen_army",
        player_level: 9,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::UndeadCrypt,
        enemy_ids: &["corpse_knight", "bone_commander"],
    },
    EncounterTemplate {
        key: "apex_predators",
        player_level: 9,
        theme: EncounterTheme::Mixed,
        background: EncounterBackground::CaveNest,
        enemy_ids: &["troll", "cave_stalker"],
    },
    EncounterTemplate {
        key: "profane_ground",
        player_level: 9,
        theme: EncounterTheme::Mixed,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["husk_whisperer", "specter"],
    },
    EncounterTemplate {
        key: "wight_siege",
        player_level: 9,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Graveyard,
        enemy_ids: &["wight", "wight"],
    },
];

// ══════════════════════════════════════════════════════════════════════════
// Level 11-12 — Final Descent (Fights 40-49)
// Peak difficulty. Mini-bosses. Factions at full strength.
// ══════════════════════════════════════════════════════════════════════════

const LEVEL_11_ENCOUNTERS: &[EncounterTemplate] = &[
    EncounterTemplate {
        key: "necromancer_lord",
        player_level: 11,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::UndeadCrypt,
        enemy_ids: &["necromancer", "corpse_knight"],
    },
    EncounterTemplate {
        key: "double_necromancer",
        player_level: 11,
        theme: EncounterTheme::Undead,
        background: EncounterBackground::Crypt,
        enemy_ids: &["necromancer", "wight"],
    },
    EncounterTemplate {
        key: "cult_apocalypse",
        player_level: 11,
        theme: EncounterTheme::Cult,
        background: EncounterBackground::CultSanctum,
        enemy_ids: &["high_cultist", "dark_priest"],
    },
    EncounterTemplate {
        key: "cult_demon",
        player_level: 11,
        theme: EncounterTheme::Fiends,
        background: EncounterBackground::FiendGate,
        enemy_ids: &["high_cultist", "flamewrath"],
    },
    EncounterTemplate {
        key: "infernal_breach",
        player_level: 11,
        theme: EncounterTheme::Fiends,
        background: EncounterBackground::FiendGate,
        enemy_ids: &["flamewrath", "flamewrath"],
    },
    EncounterTemplate {
        key: "construct_colossus",
        player_level: 11,
        theme: EncounterTheme::Constructs,
        background: EncounterBackground::AncientVault,
        enemy_ids: &["iron_golem", "iron_golem"],
    },
    EncounterTemplate {
        key: "draconic_fury",
        player_level: 11,
        theme: EncounterTheme::Beasts,
        background: EncounterBackground::DragonApproach,
        enemy_ids: &["wyvern", "young_drake"],
    },
    EncounterTemplate {
        key: "unhallowed_legion",
        player_level: 11,
        theme: EncounterTheme::Mixed,
        background: EncounterBackground::UndeadCrypt,
        enemy_ids: &["corpse_knight", "flamewrath"],
    },
    EncounterTemplate {
        key: "final_mercenaries",
        player_level: 11,
        theme: EncounterTheme::Bandits,
        background: EncounterBackground::MercenaryFort,
        enemy_ids: &["sellsword_captain", "dark_priest"],
    },
    EncounterTemplate {
        key: "flamewrath_wyvern",
        player_level: 11,
        theme: EncounterTheme::Mixed,
        background: EncounterBackground::DragonApproach,
        enemy_ids: &["flamewrath", "wyvern"],
    },
];

pub const MAX_ENEMIES: usize = 4;

// ── Public accessors ─────────────────────────────────────────────────────

pub fn all_curated_templates() -> Vec<&'static [EncounterTemplate]> {
    vec![
        LEVEL_1_ENCOUNTERS,
        LEVEL_2_ENCOUNTERS,
        LEVEL_3_ENCOUNTERS,
        LEVEL_4_ENCOUNTERS,
        LEVEL_5_ENCOUNTERS,
        LEVEL_7_ENCOUNTERS,
        LEVEL_9_ENCOUNTERS,
        LEVEL_11_ENCOUNTERS,
    ]
}

pub fn encounter_table_hash_inputs() -> Vec<String> {
    all_curated_templates()
        .into_iter()
        .flat_map(|table| table.iter().map(|t| t.hash_input()))
        .collect()
}

/// Generate a level-aware encounter for the given player level.
///
/// Levels 1-12 use curated tables with explicit themes and backgrounds.
/// Higher levels fall back to the procedural budget system.
pub fn encounter_for_level(
    player_level: u32,
    fight_index: usize,
    act: u32,
    rng: &mut GameRng,
) -> Vec<EnemyId> {
    if let Some(templates) = curated_templates_for_level(player_level) {
        let idx = rng.range(0, (templates.len() as i32) - 1) as usize;
        return templates[idx].to_enemy_ids();
    }
    gauntlet_encounter(fight_index, act, rng)
}

/// Generate a gauntlet encounter for the given fight index (0-based within
/// the act) and act number. Returns a list of enemy IDs to spawn.
pub fn gauntlet_encounter(fight_index: usize, act: u32, rng: &mut GameRng) -> Vec<EnemyId> {
    let act_bonus: i32 = match act {
        1 => 0,
        2 => 1,
        _ => 2,
    };

    let budget = match fight_index {
        0..=3 => rng.range(2, 3) + act_bonus,
        4..=7 => rng.range(3, 5) + act_bonus,
        _ => rng.range(4, 6) + act_bonus,
    };

    fill_encounter(budget, act, rng)
}

/// Generate a boss encounter for the given act. Returns a single boss
/// enemy ID (bosses are scaled separately via `scale_boss_def`).
pub fn boss_encounter(act: u32) -> Vec<EnemyId> {
    vec![enemies::boss_for_act(act)]
}

/// Hand-crafted early encounters for the first few Gauntlet fights.
/// These expose the curated level-1 templates deterministically by index.
pub fn early_encounter(index: usize) -> Vec<EnemyId> {
    let template_index = index % LEVEL_1_ENCOUNTERS.len();
    LEVEL_1_ENCOUNTERS[template_index].to_enemy_ids()
}

// ── Internal ─────────────────────────────────────────────────────────────

fn curated_templates_for_level(player_level: u32) -> Option<&'static [EncounterTemplate]> {
    match player_level {
        0 | 1 => Some(LEVEL_1_ENCOUNTERS),
        2 => Some(LEVEL_2_ENCOUNTERS),
        3 => Some(LEVEL_3_ENCOUNTERS),
        4 => Some(LEVEL_4_ENCOUNTERS),
        5 | 6 => Some(LEVEL_5_ENCOUNTERS),
        7 | 8 => Some(LEVEL_7_ENCOUNTERS),
        9 | 10 => Some(LEVEL_9_ENCOUNTERS),
        11 | 12 => Some(LEVEL_11_ENCOUNTERS),
        _ => None,
    }
}

fn fill_encounter(budget: i32, act: u32, rng: &mut GameRng) -> Vec<EnemyId> {
    let minions = enemies::minion_ids();
    let standards = enemies::standard_ids();
    let tough = enemies::tough_ids();

    let mut enemy_ids = Vec::new();
    let mut remaining = budget;

    // Try tough first (cost 4), only in act 2+
    if remaining >= 4 && act >= 2 && rng.range(0, 99) < 40 && enemy_ids.len() < MAX_ENEMIES {
        let idx = rng.range(0, (tough.len() as i32) - 1) as usize;
        enemy_ids.push(tough[idx].clone());
        remaining -= 4;
    }

    // Fill with standards (cost 2)
    while remaining >= 2 && rng.range(0, 99) < 60 && enemy_ids.len() < MAX_ENEMIES {
        let idx = rng.range(0, (standards.len() as i32) - 1) as usize;
        enemy_ids.push(standards[idx].clone());
        remaining -= 2;
    }

    // Fill remainder with minions (cost 1)
    while remaining >= 1 && enemy_ids.len() < MAX_ENEMIES {
        let idx = rng.range(0, (minions.len() as i32) - 1) as usize;
        enemy_ids.push(minions[idx].clone());
        remaining -= 1;
    }

    // Upgrade pass: spend leftover budget by swapping cheaper enemies for stronger ones.
    // Upgrade minions → standards (1 extra budget each)
    while remaining >= 1 {
        if let Some(pos) = enemy_ids.iter().position(|id| minions.contains(id)) {
            let idx = rng.range(0, (standards.len() as i32) - 1) as usize;
            enemy_ids[pos] = standards[idx].clone();
            remaining -= 1;
        } else {
            break;
        }
    }

    // Upgrade standards → toughs (2 extra budget each, act 2+ only)
    while remaining >= 2 && act >= 2 {
        if let Some(pos) = enemy_ids.iter().position(|id| standards.contains(id)) {
            let idx = rng.range(0, (tough.len() as i32) - 1) as usize;
            enemy_ids[pos] = tough[idx].clone();
            remaining -= 2;
        } else {
            break;
        }
    }

    // Fallback: at least one enemy
    if enemy_ids.is_empty() {
        let idx = rng.range(0, (standards.len() as i32) - 1) as usize;
        enemy_ids.push(standards[idx].clone());
    }

    enemy_ids
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn assert_templates_valid(
        templates: &[EncounterTemplate],
        expected_level: u32,
        label: &str,
        all_defs: &std::collections::HashMap<EnemyId, decker_engine::enemy::EnemyDef>,
    ) {
        let mut keys = HashSet::new();
        for template in templates {
            assert_eq!(template.player_level, expected_level);
            assert!(
                keys.insert(template.key),
                "duplicate {} encounter key {:?}",
                label,
                template.key
            );
            assert!(
                !template.enemy_ids.is_empty(),
                "{} encounter {:?} should not be empty",
                label,
                template.key
            );
            assert!(
                template.enemy_ids.len() <= MAX_ENEMIES,
                "{} encounter {:?} exceeds enemy cap",
                label,
                template.key
            );
            for id in template.enemy_ids {
                assert!(
                    all_defs.contains_key(*id),
                    "{} encounter {:?} has unknown enemy {:?}",
                    label,
                    template.key,
                    id
                );
            }
        }
    }

    #[test]
    fn all_curated_templates_valid() {
        let all_defs = enemies::all_enemy_defs();
        for (level, templates) in [
            (1, LEVEL_1_ENCOUNTERS),
            (2, LEVEL_2_ENCOUNTERS),
            (3, LEVEL_3_ENCOUNTERS),
            (4, LEVEL_4_ENCOUNTERS),
            (5, LEVEL_5_ENCOUNTERS),
            (7, LEVEL_7_ENCOUNTERS),
            (9, LEVEL_9_ENCOUNTERS),
            (11, LEVEL_11_ENCOUNTERS),
        ] {
            assert_templates_valid(templates, level, &format!("level-{}", level), &all_defs);
        }
    }

    #[test]
    fn gauntlet_encounter_non_empty() {
        let mut rng = GameRng::new(42);
        for fight in 0..12 {
            let enc = gauntlet_encounter(fight, 1, &mut rng);
            assert!(!enc.is_empty(), "fight {} produced empty encounter", fight);
        }
    }

    #[test]
    fn boss_encounter_single_enemy() {
        let enc = boss_encounter(1);
        assert_eq!(enc.len(), 1);
        assert_eq!(enc[0], "ogre_warchief");
    }

    #[test]
    fn early_encounters_valid() {
        let all_defs = enemies::all_enemy_defs();
        for i in 0..LEVEL_1_ENCOUNTERS.len() {
            let enc = early_encounter(i);
            assert!(!enc.is_empty());
            for id in &enc {
                assert!(
                    all_defs.contains_key(id),
                    "early encounter {} has unknown enemy {:?}",
                    i,
                    id
                );
            }
        }
    }

    #[test]
    fn encounter_for_level_routes_correctly() {
        // Level 1 should use level 1 table
        let mut rng = GameRng::new(7);
        let allowed_1: Vec<Vec<EnemyId>> = LEVEL_1_ENCOUNTERS
            .iter()
            .map(|t| t.to_enemy_ids())
            .collect();
        for _ in 0..20 {
            let enc = encounter_for_level(1, 0, 1, &mut rng);
            assert!(allowed_1.contains(&enc), "unexpected level-1 encounter: {:?}", enc);
        }

        // Level 4 should use level 4 table
        let allowed_4: Vec<Vec<EnemyId>> = LEVEL_4_ENCOUNTERS
            .iter()
            .map(|t| t.to_enemy_ids())
            .collect();
        for _ in 0..20 {
            let enc = encounter_for_level(4, 12, 2, &mut rng);
            assert!(allowed_4.contains(&enc), "unexpected level-4 encounter: {:?}", enc);
        }

        // Level 11 should use level 11 table
        let allowed_11: Vec<Vec<EnemyId>> = LEVEL_11_ENCOUNTERS
            .iter()
            .map(|t| t.to_enemy_ids())
            .collect();
        for _ in 0..20 {
            let enc = encounter_for_level(11, 40, 3, &mut rng);
            assert!(allowed_11.contains(&enc), "unexpected level-11 encounter: {:?}", enc);
        }
    }

    #[test]
    fn encounter_for_high_level_falls_through_to_procedural() {
        let mut rng = GameRng::new(99);
        // Level 13+ should use the procedural system, not panic
        let enc = encounter_for_level(13, 45, 3, &mut rng);
        assert!(!enc.is_empty());
        assert!(enc.len() <= MAX_ENEMIES);
    }

    #[test]
    fn level_one_random_selection_can_hit_all_templates() {
        let mut seen = HashSet::new();
        let allowed: Vec<Vec<EnemyId>> = LEVEL_1_ENCOUNTERS
            .iter()
            .map(|template| template.to_enemy_ids())
            .collect();
        for seed in 0..1024 {
            let mut rng = GameRng::new(seed);
            let enc = encounter_for_level(1, 0, 1, &mut rng);
            if let Some(pos) = allowed.iter().position(|candidate| *candidate == enc) {
                seen.insert(pos);
            }
            if seen.len() == allowed.len() {
                break;
            }
        }
        assert_eq!(seen.len(), allowed.len());
    }

    #[test]
    fn encounter_table_hash_inputs_non_empty() {
        let inputs = encounter_table_hash_inputs();
        let total: usize = all_curated_templates().iter().map(|t| t.len()).sum();
        assert_eq!(inputs.len(), total);
        assert!(inputs.iter().all(|s| !s.is_empty()));
    }

    #[test]
    fn budget_scales_with_act() {
        let mut rng1 = GameRng::new(123);
        let mut rng2 = GameRng::new(123);

        let enc_act1 = gauntlet_encounter(5, 1, &mut rng1);
        let enc_act3 = gauntlet_encounter(5, 3, &mut rng2);

        assert!(!enc_act1.is_empty());
        assert!(!enc_act3.is_empty());
        assert!(enc_act1.len() <= MAX_ENEMIES);
        assert!(enc_act3.len() <= MAX_ENEMIES);
    }

    #[test]
    fn encounter_respects_max_enemies() {
        for seed in 0..100 {
            let mut rng = GameRng::new(seed);
            for act in 1..=3 {
                for fight in 0..12 {
                    let enc = gauntlet_encounter(fight, act, &mut rng);
                    assert!(
                        enc.len() <= MAX_ENEMIES,
                        "seed={} act={} fight={} produced {} enemies (max {})",
                        seed, act, fight, enc.len(), MAX_ENEMIES,
                    );
                }
            }
        }
    }

    #[test]
    fn early_encounters_within_cap() {
        for i in 0..LEVEL_1_ENCOUNTERS.len() {
            let enc = early_encounter(i);
            assert!(
                enc.len() <= MAX_ENEMIES,
                "early encounter {} has {} enemies (max {})",
                i, enc.len(), MAX_ENEMIES,
            );
        }
    }
}
