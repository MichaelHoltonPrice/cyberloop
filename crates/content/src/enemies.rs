//! Enemy definitions for the core set (all tiers).

use std::collections::HashMap;

use decker_engine::enemy::{EnemyDef, IntentType};
use decker_engine::ids::EnemyId;
use decker_engine::status::StatusType;

// ═══════════════════════════════════════════════════════════════════════════
// Minions (budget 1)
// ═══════════════════════════════════════════════════════════════════════════

pub fn goblin_grunt_def() -> EnemyDef {
    EnemyDef {
        id: "goblin_grunt".into(),
        name: "Goblin Grunt".into(),
        max_hp: 55,
        intent_pattern: vec![
            IntentType::Attack(12),
            IntentType::Attack(14),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(12),
        ],
    }
}

pub fn goblin_archer_def() -> EnemyDef {
    EnemyDef {
        id: "goblin_archer".into(),
        name: "Goblin Archer".into(),
        max_hp: 35,
        intent_pattern: vec![
            IntentType::Attack(9),
            IntentType::Attack(10),
            IntentType::Buff(StatusType::Empowered, 1),
            IntentType::Attack(9),
        ],
    }
}

pub fn goblin_shaman_def() -> EnemyDef {
    EnemyDef {
        id: "goblin_shaman".into(),
        name: "Goblin Shaman".into(),
        max_hp: 30,
        intent_pattern: vec![
            IntentType::BuffAllies(StatusType::Empowered, 1),
            IntentType::Attack(8),
            IntentType::Debuff(StatusType::Threatened, 1),
            IntentType::BuffAllies(StatusType::Empowered, 1),
            IntentType::Attack(8),
        ],
    }
}

pub fn giant_rat_def() -> EnemyDef {
    EnemyDef {
        id: "giant_rat".into(),
        name: "Giant Rat".into(),
        max_hp: 12,
        intent_pattern: vec![
            IntentType::Attack(6),
            IntentType::Attack(8),
            IntentType::Attack(6),
        ],
    }
}

pub fn scavenger_def() -> EnemyDef {
    EnemyDef {
        id: "scavenger".into(),
        name: "Scavenger".into(),
        max_hp: 60,
        intent_pattern: vec![
            IntentType::Attack(13),
            IntentType::AttackDefend(11, 6),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(13),
        ],
    }
}

pub fn fire_beetle_def() -> EnemyDef {
    EnemyDef {
        id: "fire_beetle".into(),
        name: "Fire Beetle".into(),
        max_hp: 25,
        intent_pattern: vec![
            IntentType::Attack(9),
            IntentType::Debuff(StatusType::Threatened, 1),
            IntentType::Attack(12),
        ],
    }
}

pub fn animated_shield_def() -> EnemyDef {
    EnemyDef {
        id: "animated_shield".into(),
        name: "Animated Shield".into(),
        max_hp: 55,
        intent_pattern: vec![
            IntentType::Defend(10),
            IntentType::Buff(StatusType::Empowered, 1),
            IntentType::Attack(12),
            IntentType::AttackDefend(10, 8),
        ],
    }
}

pub fn sporecap_def() -> EnemyDef {
    EnemyDef {
        id: "sporecap".into(),
        name: "Sporecap".into(),
        max_hp: 30,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Weakened, 1),
            IntentType::Attack(8),
            IntentType::BuffAllies(StatusType::Empowered, 1),
            IntentType::Attack(10),
        ],
    }
}

pub fn skeleton_def() -> EnemyDef {
    EnemyDef {
        id: "skeleton".into(),
        name: "Skeleton".into(),
        max_hp: 22,
        intent_pattern: vec![
            IntentType::Attack(9),
            IntentType::Attack(11),
            IntentType::Buff(StatusType::Empowered, 1),
        ],
    }
}

pub fn blood_moth_def() -> EnemyDef {
    EnemyDef {
        id: "blood_moth".into(),
        name: "Blood Moth".into(),
        max_hp: 14,
        intent_pattern: vec![
            IntentType::Attack(7),
            IntentType::Debuff(StatusType::Bleeding, 2),
            IntentType::Attack(9),
        ],
    }
}

pub fn bandit_thug_def() -> EnemyDef {
    EnemyDef {
        id: "bandit_thug".into(),
        name: "Bandit Thug".into(),
        max_hp: 70,
        intent_pattern: vec![
            IntentType::Attack(15),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(15),
            IntentType::AttackDefend(13, 8),
        ],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Standard (budget 2)
// ═══════════════════════════════════════════════════════════════════════════

pub fn slime_def() -> EnemyDef {
    EnemyDef {
        id: "slime".into(),
        name: "Slime".into(),
        max_hp: 60,
        intent_pattern: vec![IntentType::Attack(10), IntentType::Defend(10)],
    }
}

pub fn zombie_def() -> EnemyDef {
    EnemyDef {
        id: "zombie".into(),
        name: "Zombie".into(),
        max_hp: 50,
        intent_pattern: vec![
            IntentType::Attack(8),
            IntentType::Defend(6),
            IntentType::Attack(10),
        ],
    }
}

pub fn plague_rat_def() -> EnemyDef {
    EnemyDef {
        id: "plague_rat".into(),
        name: "Plague Rat".into(),
        max_hp: 30,
        intent_pattern: vec![
            IntentType::Attack(8),
            IntentType::Debuff(StatusType::Bleeding, 2),
            IntentType::Attack(10),
        ],
    }
}

pub fn cave_spider_def() -> EnemyDef {
    EnemyDef {
        id: "cave_spider".into(),
        name: "Cave Spider".into(),
        max_hp: 35,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Weakened, 1),
            IntentType::Attack(10),
            IntentType::Attack(12),
        ],
    }
}

pub fn giant_centipede_def() -> EnemyDef {
    EnemyDef {
        id: "giant_centipede".into(),
        name: "Giant Centipede".into(),
        max_hp: 40,
        intent_pattern: vec![
            IntentType::Attack(9),
            IntentType::Debuff(StatusType::Bleeding, 2),
            IntentType::Attack(11),
        ],
    }
}

pub fn bandit_archer_def() -> EnemyDef {
    EnemyDef {
        id: "bandit_archer".into(),
        name: "Bandit Archer".into(),
        max_hp: 35,
        intent_pattern: vec![
            IntentType::Attack(10),
            IntentType::Attack(12),
            IntentType::Attack(10),
            IntentType::Debuff(StatusType::Marked, 1),
        ],
    }
}

pub fn imp_def() -> EnemyDef {
    EnemyDef {
        id: "imp".into(),
        name: "Imp".into(),
        max_hp: 45,
        intent_pattern: vec![
            IntentType::Attack(10),
            IntentType::Attack(10),
            IntentType::Buff(StatusType::Empowered, 2),
        ],
    }
}

pub fn shadow_wisp_def() -> EnemyDef {
    EnemyDef {
        id: "shadow_wisp".into(),
        name: "Shadow Wisp".into(),
        max_hp: 35,
        intent_pattern: vec![
            IntentType::Attack(9),
            IntentType::Attack(9),
            IntentType::Debuff(StatusType::Weakened, 1),
        ],
    }
}

pub fn bone_sentinel_def() -> EnemyDef {
    EnemyDef {
        id: "bone_sentinel".into(),
        name: "Bone Sentinel".into(),
        max_hp: 70,
        intent_pattern: vec![
            IntentType::Defend(12),
            IntentType::Attack(14),
            IntentType::AttackDefend(11, 8),
        ],
    }
}

pub fn harpy_def() -> EnemyDef {
    EnemyDef {
        id: "harpy".into(),
        name: "Harpy".into(),
        max_hp: 50,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Weakened, 2),
            IntentType::Attack(13),
            IntentType::Attack(13),
            IntentType::Debuff(StatusType::Threatened, 1),
        ],
    }
}

pub fn specter_def() -> EnemyDef {
    EnemyDef {
        id: "specter".into(),
        name: "Specter".into(),
        max_hp: 45,
        intent_pattern: vec![
            IntentType::Attack(11),
            IntentType::Debuff(StatusType::Marked, 2),
            IntentType::Attack(11),
            IntentType::Debuff(StatusType::Bleeding, 3),
        ],
    }
}

pub fn rust_crawler_def() -> EnemyDef {
    EnemyDef {
        id: "rust_crawler".into(),
        name: "Rust Crawler".into(),
        max_hp: 55,
        intent_pattern: vec![
            IntentType::AttackDefend(11, 8),
            IntentType::Debuff(StatusType::Weakened, 1),
            IntentType::AttackDefend(11, 8),
            IntentType::Buff(StatusType::Empowered, 1),
        ],
    }
}

pub fn blightcap_def() -> EnemyDef {
    EnemyDef {
        id: "blightcap".into(),
        name: "Blightcap".into(),
        max_hp: 50,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Threatened, 1),
            IntentType::Attack(11),
            IntentType::Debuff(StatusType::Weakened, 1),
            IntentType::AttackDefend(10, 6),
        ],
    }
}

pub fn wolf_def() -> EnemyDef {
    EnemyDef {
        id: "wolf".into(),
        name: "Wolf".into(),
        max_hp: 30,
        intent_pattern: vec![
            IntentType::Attack(10),
            IntentType::Debuff(StatusType::Marked, 1),
            IntentType::Attack(13),
        ],
    }
}

pub fn boar_def() -> EnemyDef {
    EnemyDef {
        id: "boar".into(),
        name: "Boar".into(),
        max_hp: 45,
        intent_pattern: vec![
            IntentType::Attack(16),
            IntentType::Defend(8),
            IntentType::Attack(10),
        ],
    }
}

pub fn cult_acolyte_def() -> EnemyDef {
    EnemyDef {
        id: "cult_acolyte".into(),
        name: "Cult Acolyte".into(),
        max_hp: 40,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Weakened, 1),
            IntentType::Attack(10),
            IntentType::Debuff(StatusType::Threatened, 1),
            IntentType::Attack(12),
        ],
    }
}

pub fn wight_def() -> EnemyDef {
    EnemyDef {
        id: "wight".into(),
        name: "Wight".into(),
        max_hp: 50,
        intent_pattern: vec![
            IntentType::Attack(12),
            IntentType::Debuff(StatusType::Weakened, 1),
            IntentType::Attack(14),
            IntentType::Buff(StatusType::Mending, 3),
        ],
    }
}

pub fn sellsword_def() -> EnemyDef {
    EnemyDef {
        id: "sellsword".into(),
        name: "Sellsword".into(),
        max_hp: 50,
        intent_pattern: vec![
            IntentType::Attack(13),
            IntentType::Defend(10),
            IntentType::Attack(15),
            IntentType::AttackDefend(10, 8),
        ],
    }
}

pub fn young_drake_def() -> EnemyDef {
    EnemyDef {
        id: "young_drake".into(),
        name: "Young Drake".into(),
        max_hp: 45,
        intent_pattern: vec![
            IntentType::Attack(12),
            IntentType::Attack(14),
            IntentType::Debuff(StatusType::Threatened, 1),
        ],
    }
}

pub fn animated_armor_def() -> EnemyDef {
    EnemyDef {
        id: "animated_armor".into(),
        name: "Animated Armor".into(),
        max_hp: 45,
        intent_pattern: vec![
            IntentType::Attack(13),
            IntentType::AttackDefend(10, 8),
            IntentType::Attack(15),
        ],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tough (budget 4)
// ═══════════════════════════════════════════════════════════════════════════

pub fn ogre_thug_def() -> EnemyDef {
    EnemyDef {
        id: "ogre_thug".into(),
        name: "Ogre Thug".into(),
        max_hp: 55,
        intent_pattern: vec![
            IntentType::Attack(12),
            IntentType::Attack(12),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(12),
        ],
    }
}

pub fn stone_golem_def() -> EnemyDef {
    EnemyDef {
        id: "stone_golem".into(),
        name: "Stone Golem".into(),
        max_hp: 90,
        intent_pattern: vec![
            IntentType::Attack(15),
            IntentType::Defend(12),
            IntentType::AttackDefend(12, 8),
            IntentType::Buff(StatusType::Empowered, 1),
        ],
    }
}

pub fn cave_stalker_def() -> EnemyDef {
    EnemyDef {
        id: "cave_stalker".into(),
        name: "Cave Stalker".into(),
        max_hp: 70,
        intent_pattern: vec![
            IntentType::Attack(14),
            IntentType::Attack(14),
            IntentType::AttackDefend(10, 8),
            IntentType::Buff(StatusType::Empowered, 2),
        ],
    }
}

pub fn husk_whisperer_def() -> EnemyDef {
    EnemyDef {
        id: "husk_whisperer".into(),
        name: "Husk Whisperer".into(),
        max_hp: 55,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Threatened, 2),
            IntentType::Attack(10),
            IntentType::Debuff(StatusType::Weakened, 2),
            IntentType::Attack(10),
            IntentType::Debuff(StatusType::Marked, 3),
        ],
    }
}

pub fn troll_def() -> EnemyDef {
    EnemyDef {
        id: "troll".into(),
        name: "Troll".into(),
        max_hp: 75,
        intent_pattern: vec![
            IntentType::Attack(11),
            IntentType::Buff(StatusType::Mending, 3),
            IntentType::Attack(14),
            IntentType::AttackDefend(11, 6),
        ],
    }
}

pub fn rust_crawler_tough_def() -> EnemyDef {
    // Rust crawler is standard tier, but we keep it there.
    // This is the tough-tier version used for procedural upgrades.
    // (Not used directly — standard rust_crawler fills the role.)
    rust_crawler_def()
}

pub fn bone_commander_def() -> EnemyDef {
    EnemyDef {
        id: "bone_commander".into(),
        name: "Bone Commander".into(),
        max_hp: 58,
        intent_pattern: vec![
            IntentType::BuffAllies(StatusType::Empowered, 2),
            IntentType::Attack(10),
            IntentType::Defend(8),
            IntentType::Attack(12),
        ],
    }
}

pub fn cult_channeler_def() -> EnemyDef {
    EnemyDef {
        id: "cult_channeler".into(),
        name: "Cult Channeler".into(),
        max_hp: 52,
        intent_pattern: vec![
            IntentType::BuffAllies(StatusType::Empowered, 1),
            IntentType::Attack(9),
            IntentType::Debuff(StatusType::Weakened, 2),
            IntentType::Attack(10),
        ],
    }
}

pub fn cult_invoker_def() -> EnemyDef {
    EnemyDef {
        id: "cult_invoker".into(),
        name: "Cult Invoker".into(),
        max_hp: 48,
        intent_pattern: vec![
            IntentType::Attack(11),
            IntentType::Debuff(StatusType::Threatened, 1),
            IntentType::Attack(13),
            IntentType::Defend(6),
        ],
    }
}

pub fn dark_priest_def() -> EnemyDef {
    EnemyDef {
        id: "dark_priest".into(),
        name: "Dark Priest".into(),
        max_hp: 50,
        intent_pattern: vec![
            IntentType::BuffAllies(StatusType::Mending, 2),
            IntentType::Debuff(StatusType::Threatened, 2),
            IntentType::Attack(9),
            IntentType::BuffAllies(StatusType::Mending, 2),
            IntentType::Attack(10),
        ],
    }
}

pub fn corpse_knight_def() -> EnemyDef {
    EnemyDef {
        id: "corpse_knight".into(),
        name: "Corpse Knight".into(),
        max_hp: 62,
        intent_pattern: vec![
            IntentType::Attack(13),
            IntentType::AttackDefend(10, 6),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(15),
        ],
    }
}

pub fn sellsword_captain_def() -> EnemyDef {
    EnemyDef {
        id: "sellsword_captain".into(),
        name: "Sellsword Captain".into(),
        max_hp: 56,
        intent_pattern: vec![
            IntentType::BuffAllies(StatusType::Empowered, 2),
            IntentType::Attack(10),
            IntentType::AttackDefend(8, 6),
            IntentType::Attack(12),
        ],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Elite (placed directly)
// ═══════════════════════════════════════════════════════════════════════════

pub fn flamewrath_def() -> EnemyDef {
    EnemyDef {
        id: "flamewrath".into(),
        name: "Flamewrath".into(),
        max_hp: 80,
        intent_pattern: vec![
            IntentType::Attack(13),
            IntentType::Debuff(StatusType::Bleeding, 3),
            IntentType::AttackDefend(10, 10),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(13),
            IntentType::Debuff(StatusType::Threatened, 2),
        ],
    }
}

pub fn wyvern_def() -> EnemyDef {
    EnemyDef {
        id: "wyvern".into(),
        name: "Wyvern".into(),
        max_hp: 85,
        intent_pattern: vec![
            IntentType::Attack(15),
            IntentType::Defend(12),
            IntentType::Attack(18),
            IntentType::Debuff(StatusType::Bleeding, 4),
            IntentType::AttackDefend(12, 8),
        ],
    }
}

pub fn necromancer_def() -> EnemyDef {
    EnemyDef {
        id: "necromancer".into(),
        name: "Necromancer".into(),
        max_hp: 70,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Marked, 3),
            IntentType::BuffAllies(StatusType::Empowered, 2),
            IntentType::Attack(12),
            IntentType::Debuff(StatusType::Threatened, 2),
            IntentType::Attack(12),
            IntentType::Debuff(StatusType::Weakened, 2),
        ],
    }
}

pub fn high_cultist_def() -> EnemyDef {
    EnemyDef {
        id: "high_cultist".into(),
        name: "High Cultist".into(),
        max_hp: 72,
        intent_pattern: vec![
            IntentType::BuffAllies(StatusType::Empowered, 3),
            IntentType::Debuff(StatusType::Threatened, 2),
            IntentType::Attack(11),
            IntentType::Debuff(StatusType::Weakened, 2),
            IntentType::Attack(13),
        ],
    }
}

pub fn iron_golem_def() -> EnemyDef {
    EnemyDef {
        id: "iron_golem".into(),
        name: "Iron Golem".into(),
        max_hp: 95,
        intent_pattern: vec![
            IntentType::Defend(12),
            IntentType::Attack(14),
            IntentType::AttackDefend(12, 8),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(14),
            IntentType::Defend(12),
        ],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Act Bosses
// ═══════════════════════════════════════════════════════════════════════════

pub fn ogre_warchief_def() -> EnemyDef {
    EnemyDef {
        id: "ogre_warchief".into(),
        name: "Ogre Warchief".into(),
        max_hp: 80,
        intent_pattern: vec![
            IntentType::Attack(14),
            IntentType::Attack(14),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::AttackDefend(12, 8),
            IntentType::Debuff(StatusType::Threatened, 1),
            IntentType::Attack(14),
        ],
    }
}

pub fn pale_warden_def() -> EnemyDef {
    EnemyDef {
        id: "pale_warden".into(),
        name: "Pale Warden".into(),
        max_hp: 75,
        intent_pattern: vec![
            IntentType::Debuff(StatusType::Marked, 3),
            IntentType::Attack(12),
            IntentType::Debuff(StatusType::Weakened, 2),
            IntentType::AttackDefend(10, 10),
            IntentType::Debuff(StatusType::Threatened, 2),
            IntentType::Attack(15),
            IntentType::Buff(StatusType::Empowered, 2),
        ],
    }
}

pub fn dragon_wyrm_def() -> EnemyDef {
    EnemyDef {
        id: "dragon_wyrm".into(),
        name: "Dragon Wyrm".into(),
        max_hp: 85,
        intent_pattern: vec![
            IntentType::Attack(14),
            IntentType::Defend(12),
            IntentType::Debuff(StatusType::Bleeding, 3),
            IntentType::AttackDefend(12, 8),
            IntentType::Buff(StatusType::Empowered, 2),
            IntentType::Attack(16),
            IntentType::Debuff(StatusType::Threatened, 2),
            IntentType::Attack(14),
        ],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Aggregates
// ═══════════════════════════════════════════════════════════════════════════

/// Every enemy definition in the current content set.
pub fn all_enemy_defs() -> HashMap<EnemyId, EnemyDef> {
    let all = vec![
        // Minions
        goblin_grunt_def(),
        goblin_archer_def(),
        goblin_shaman_def(),
        giant_rat_def(),
        scavenger_def(),
        fire_beetle_def(),
        animated_shield_def(),
        sporecap_def(),
        skeleton_def(),
        blood_moth_def(),
        bandit_thug_def(),
        // Standard
        slime_def(),
        zombie_def(),
        plague_rat_def(),
        cave_spider_def(),
        giant_centipede_def(),
        bandit_archer_def(),
        imp_def(),
        shadow_wisp_def(),
        bone_sentinel_def(),
        harpy_def(),
        specter_def(),
        rust_crawler_def(),
        blightcap_def(),
        wolf_def(),
        boar_def(),
        cult_acolyte_def(),
        wight_def(),
        sellsword_def(),
        young_drake_def(),
        animated_armor_def(),
        // Tough
        ogre_thug_def(),
        stone_golem_def(),
        cave_stalker_def(),
        husk_whisperer_def(),
        troll_def(),
        bone_commander_def(),
        cult_channeler_def(),
        cult_invoker_def(),
        dark_priest_def(),
        corpse_knight_def(),
        sellsword_captain_def(),
        // Elite
        flamewrath_def(),
        wyvern_def(),
        necromancer_def(),
        high_cultist_def(),
        iron_golem_def(),
        // Bosses
        ogre_warchief_def(),
        pale_warden_def(),
        dragon_wyrm_def(),
    ];
    all.into_iter().map(|d| (d.id.clone(), d)).collect()
}

// ── Tier lookups ─────────────────────────────────────────────────────────

/// Minion-tier enemy IDs (low HP, ~1 budget).
pub fn minion_ids() -> Vec<EnemyId> {
    vec![
        "goblin_grunt".into(),
        "goblin_archer".into(),
        "goblin_shaman".into(),
        "giant_rat".into(),
        "scavenger".into(),
        "fire_beetle".into(),
        "animated_shield".into(),
        "sporecap".into(),
        "skeleton".into(),
        "blood_moth".into(),
        "bandit_thug".into(),
    ]
}

/// Standard-tier enemy IDs (~2 budget).
pub fn standard_ids() -> Vec<EnemyId> {
    vec![
        "slime".into(),
        "zombie".into(),
        "plague_rat".into(),
        "cave_spider".into(),
        "giant_centipede".into(),
        "bandit_archer".into(),
        "imp".into(),
        "shadow_wisp".into(),
        "bone_sentinel".into(),
        "harpy".into(),
        "specter".into(),
        "rust_crawler".into(),
        "blightcap".into(),
        "wolf".into(),
        "boar".into(),
        "cult_acolyte".into(),
        "wight".into(),
        "sellsword".into(),
        "young_drake".into(),
        "animated_armor".into(),
    ]
}

/// Tough-tier enemy IDs (~4 budget).
pub fn tough_ids() -> Vec<EnemyId> {
    vec![
        "ogre_thug".into(),
        "stone_golem".into(),
        "cave_stalker".into(),
        "husk_whisperer".into(),
        "troll".into(),
        "bone_commander".into(),
        "cult_channeler".into(),
        "cult_invoker".into(),
        "dark_priest".into(),
        "corpse_knight".into(),
        "sellsword_captain".into(),
    ]
}

/// Elite-tier enemy IDs (hard solo fights or paired).
pub fn elite_ids() -> Vec<EnemyId> {
    vec![
        "flamewrath".into(),
        "wyvern".into(),
        "necromancer".into(),
        "high_cultist".into(),
        "iron_golem".into(),
    ]
}

/// Act boss enemy IDs.
pub fn boss_ids() -> Vec<EnemyId> {
    vec![
        "ogre_warchief".into(),
        "pale_warden".into(),
        "dragon_wyrm".into(),
    ]
}

/// Boss enemy ID for a given act.
pub fn boss_for_act(act: u32) -> EnemyId {
    match act {
        1 => "ogre_warchief".into(),
        2 => "pale_warden".into(),
        _ => "dragon_wyrm".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_defs_count() {
        let defs = all_enemy_defs();
        assert_eq!(defs.len(), 50);
    }

    #[test]
    fn no_duplicate_ids() {
        let all = all_enemy_defs();
        // all_enemy_defs() uses a HashMap so duplicate IDs would silently
        // overwrite — compare against the vec length.
        let all_vec: Vec<EnemyDef> = vec![
            goblin_grunt_def(), goblin_archer_def(), goblin_shaman_def(),
            giant_rat_def(), scavenger_def(), fire_beetle_def(),
            animated_shield_def(), sporecap_def(), skeleton_def(),
            blood_moth_def(), bandit_thug_def(),
            slime_def(), zombie_def(), plague_rat_def(), cave_spider_def(),
            giant_centipede_def(), bandit_archer_def(), imp_def(),
            shadow_wisp_def(), bone_sentinel_def(), harpy_def(), specter_def(),
            rust_crawler_def(), blightcap_def(), wolf_def(), boar_def(),
            cult_acolyte_def(), wight_def(), sellsword_def(), young_drake_def(),
            animated_armor_def(),
            ogre_thug_def(), stone_golem_def(), cave_stalker_def(),
            husk_whisperer_def(), troll_def(), bone_commander_def(),
            cult_channeler_def(), cult_invoker_def(), dark_priest_def(),
            corpse_knight_def(), sellsword_captain_def(),
            flamewrath_def(), wyvern_def(), necromancer_def(), high_cultist_def(),
            iron_golem_def(),
            ogre_warchief_def(), pale_warden_def(), dragon_wyrm_def(),
        ];
        assert_eq!(all_vec.len(), all.len(), "duplicate enemy IDs detected");
    }

    #[test]
    fn boss_for_all_acts() {
        assert_eq!(boss_for_act(1), "ogre_warchief");
        assert_eq!(boss_for_act(2), "pale_warden");
        assert_eq!(boss_for_act(3), "dragon_wyrm");
    }

    #[test]
    fn all_tier_ids_exist_in_defs() {
        let defs = all_enemy_defs();
        for id in minion_ids()
            .iter()
            .chain(standard_ids().iter())
            .chain(tough_ids().iter())
            .chain(elite_ids().iter())
            .chain(boss_ids().iter())
        {
            assert!(
                defs.contains_key(id),
                "tier ID {:?} not found in all_enemy_defs",
                id
            );
        }
    }

    #[test]
    fn every_enemy_has_at_least_one_attack() {
        let defs = all_enemy_defs();
        for (id, def) in &defs {
            let has_attack = def.intent_pattern.iter().any(|intent| {
                matches!(intent, IntentType::Attack(_) | IntentType::AttackDefend(_, _))
            });
            assert!(
                has_attack,
                "enemy {:?} has no Attack or AttackDefend in intent pattern",
                id
            );
        }
    }
}
