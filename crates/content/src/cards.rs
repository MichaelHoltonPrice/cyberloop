//! Card definitions for the Decker engine.
//!
//! This is the single source of truth for all card data. Cards are defined
//! as functions returning `CardDef` and aggregated by `all_card_defs()`.

use std::collections::HashMap;

use decker_engine::card::*;
use decker_engine::card_ids::*;
use decker_engine::class::{CardTag, Class};
use decker_engine::status::StatusType;

/// Training-only switch for reintroducing the temporary overpowered reward card.
///
/// Enabled via the Cargo feature `op-test-card`. The card definition stays in
/// content regardless so observation/model shapes remain stable; this flag only
/// controls whether it is offered in reward pools.
pub const OP_TEST_CARD_ENABLED: bool = cfg!(feature = "op-test-card");

// ── Helper constructors ────────────────────────────────────────────────

fn universal(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: None,
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn fighter(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Fighter),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn fighter_sc(
    id: &str,
    name: &str,
    subclass: &str,
    cost: i32,
    effects: Vec<Effect>,
    desc: &str,
) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Fighter),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn barbarian(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Barbarian),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn barbarian_sc(
    id: &str,
    name: &str,
    subclass: &str,
    cost: i32,
    effects: Vec<Effect>,
    desc: &str,
) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Barbarian),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn monk(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Monk),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn monk_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Monk),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn paladin(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Paladin),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn paladin_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Paladin),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn wizard(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Wizard),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn sorcerer(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Sorcerer),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn sorcerer_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Sorcerer),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn wizard_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Wizard),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

// ── Core basics ────────────────────────────────────────────────────────

fn strike_def() -> CardDef {
    let mut d = universal(
        STRIKE,
        "Strike",
        1,
        vec![Effect::DealDamage {
            target: Target::SingleEnemy,
            amount: 10,
        }],
        "Deal 10 damage.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn defend_def() -> CardDef {
    let mut d = universal(
        DEFEND,
        "Defend",
        1,
        vec![Effect::GainBlock { amount: 10 }],
        "Gain 10 block.",
    );
    d.tags = vec![CardTag::Defense];
    d
}

fn wound_def() -> CardDef {
    CardDef {
        id: WOUND.into(),
        name: "Wound".into(),
        rarity: Rarity::Common,
        cost: 1,
        card_type: CardType::Spell,
        exhaust: false,
        effects: vec![],
        description: "Unplayable dead weight.".into(),
        class: None,
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

// ── Themed basics ──────────────────────────────────────────────────────

fn barbs_def() -> CardDef {
    let mut d = fighter_sc(
        BARBS,
        "Barbs",
        "defense",
        1,
        vec![
            Effect::GainBlock { amount: 10 },
            Effect::ApplyStatus {
                target: Target::Player,
                status: StatusType::Barbed,
                stacks: 1,
            },
        ],
        "Gain 10 block. Apply 1 Barbed to self.",
    );
    d.tags = vec![CardTag::Defense];
    d
}

fn heavy_strike_def() -> CardDef {
    let mut d = fighter_sc(
        HEAVY_STRIKE,
        "Heavy Strike",
        "two_handed",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::ApplyStatus {
                target: Target::SingleEnemy,
                status: StatusType::Weakened,
                stacks: 1,
            },
        ],
        "Deal 10 damage. Apply 1 Weakened.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn measured_strike_def() -> CardDef {
    let mut d = fighter_sc(
        MEASURED_STRIKE,
        "Measured Strike",
        "dueling",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::ApplyStatus {
                target: Target::SingleEnemy,
                status: StatusType::Marked,
                stacks: 2,
            },
        ],
        "Deal 10 damage. Apply 2 Marked.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn deft_strike_def() -> CardDef {
    let mut d = fighter_sc(
        DEFT_STRIKE,
        "Deft Strike",
        "dueling",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::DrawCards { count: 1 },
        ],
        "Deal 10 damage. Draw 1 card.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

// ── Fighter class cards ────────────────────────────────────────────────

fn second_wind_def() -> CardDef {
    let mut d = fighter(
        SECOND_WIND,
        "Second Wind",
        1,
        vec![Effect::Heal { amount: 15 }],
        "Heal 15 HP.",
    );
    d.card_type = CardType::Consumable;
    d.tags = vec![CardTag::Skill];
    d
}

fn energy_surge_def() -> CardDef {
    let mut d = fighter(
        ENERGY_SURGE,
        "Energy Surge",
        0,
        vec![
            Effect::GainEnergy { amount: 1 },
            Effect::GainBlock { amount: 8 },
        ],
        "Gain 1 energy. Gain 8 block. Exhaust.",
    );
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn extra_attack_def() -> CardDef {
    let mut d = fighter(
        EXTRA_ATTACK,
        "Extra Attack",
        0,
        vec![
            Effect::GainBlock { amount: 8 },
            Effect::DrawCards { count: 1 },
        ],
        "Gain 8 block. Draw 1 card. Exhaust.",
    );
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

fn rally_def() -> CardDef {
    let mut d = fighter(
        RALLY,
        "Rally",
        0,
        vec![
            Effect::GainBlock { amount: 10 },
            Effect::DrawCards { count: 1 },
        ],
        "Gain 10 block. Draw 1 card. Innate. Exhaust.",
    );
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

// ── Defense subclass progression ───────────────────────────────────────

fn iron_guard_def() -> CardDef {
    let mut d = fighter_sc(
        IRON_GUARD,
        "Iron Guard",
        "defense",
        0,
        vec![Effect::GainBlock { amount: 12 }],
        "Gain 12 block. Exhaust.",
    );
    d.exhaust = true;
    d.milestone = true;
    d.tags = vec![CardTag::Power];
    d
}

fn shield_bash_def() -> CardDef {
    let mut d = fighter_sc(
        SHIELD_BASH,
        "Shield Bash",
        "defense",
        1,
        vec![
            Effect::DealDamageEqualBlock {
                target: Target::SingleEnemy,
            },
            Effect::GainBlock { amount: 3 },
        ],
        "Deal damage equal to your block. Gain 3 block.",
    );
    d.milestone = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn brace_def() -> CardDef {
    let mut d = fighter_sc(
        BRACE,
        "Brace",
        "defense",
        1,
        vec![Effect::GainBlockConditional {
            amount: 8,
            bonus_amount: 12,
        }],
        "Gain 8 block. If you already have block, gain 12 instead.",
    );
    d.milestone = true;
    d.tags = vec![CardTag::Defense];
    d
}

fn unbreakable_def() -> CardDef {
    let mut d = fighter_sc(
        UNBREAKABLE,
        "Unbreakable",
        "defense",
        0,
        vec![Effect::BlockFloor { amount: 5 }],
        "Block cannot go below 5 this combat. Exhaust.",
    );
    d.exhaust = true;
    d.milestone = true;
    d.tags = vec![CardTag::Power];
    d
}

// ── Defense subclass: Bulwark synergy group ──────────────────────────

fn fortify_def() -> CardDef {
    let mut d = fighter_sc(FORTIFY, "Fortify", "defense", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::BlockRetention, stacks: 1 },
    ], "Gain 8 block. Your block is not reset. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

fn entrench_def() -> CardDef {
    let mut d = fighter_sc(ENTRENCH, "Entrench", "defense", 1, vec![
        Effect::GainBlockConditional { amount: 10, bonus_amount: 16 },
    ], "Gain 10 block. If you already have block, gain 16 instead.");
    d.tags = vec![CardTag::Defense];
    d
}

fn stalwart_defense_def() -> CardDef {
    let mut d = fighter_sc(STALWART_DEFENSE, "Stalwart Defense", "defense", 1, vec![
        Effect::GainBlock { amount: 6 },
        Effect::DrawCards { count: 1 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
    ], "Gain 6 block. Draw 1 card. Apply 1 Weakened.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn shield_wall_def() -> CardDef {
    let mut d = fighter_sc(SHIELD_WALL, "Shield Wall", "defense", 2, vec![
        Effect::GainBlock { amount: 20 },
        Effect::GainBlockConditional { amount: 0, bonus_amount: 6 },
    ], "Gain 20 block. If you already have block, gain 6 more.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Defense];
    d
}

fn aegis_eternal_def() -> CardDef {
    let mut d = fighter_sc(AEGIS_ETERNAL, "Aegis Eternal", "defense", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::BlockRetention, stacks: 1 },
        Effect::BlockFloor { amount: 8 },
        Effect::GainBlock { amount: 15 },
    ], "Your block is never reset. Block cannot go below 8. Gain 15 block. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

// ── Defense subclass: Reprisal synergy group ─────────────────────────

fn spiked_armor_def() -> CardDef {
    let mut d = fighter_sc(SPIKED_ARMOR, "Spiked Armor", "defense", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 2 },
    ], "Gain 8 block. Gain 2 Barbed.");
    d.tags = vec![CardTag::Defense];
    d
}

fn retribution_def() -> CardDef {
    let mut d = fighter_sc(RETRIBUTION, "Retribution", "defense", 1, vec![
        Effect::DealDamageIfDamagedLastTurn { target: Target::SingleEnemy, amount: 12 },
        Effect::GainBlockIfDamagedLastTurn { amount: 8 },
    ], "If you took damage last turn: deal 12 damage and gain 8 block.");
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn thorned_carapace_def() -> CardDef {
    let mut d = fighter_sc(THORNED_CARAPACE, "Thorned Carapace", "defense", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 3 },
    ], "Gain 3 Barbed. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

fn barbed_bulwark_def() -> CardDef {
    let mut d = fighter_sc(BARBED_BULWARK, "Barbed Bulwark", "defense", 1, vec![
        Effect::GainBlockPerStatusStack { status: StatusType::Barbed, multiplier: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 1 },
    ], "Gain block equal to 3x your Barbed stacks. Gain 1 Barbed.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Defense];
    d
}

fn wrath_of_thorns_def() -> CardDef {
    let mut d = fighter_sc(WRATH_OF_THORNS, "Wrath of Thorns", "defense", 1, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 4 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 2 },
        Effect::GainBlock { amount: 10 },
    ], "Gain 4 Barbed. Apply 2 Threatened to ALL enemies. Gain 10 block. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

// ── Two-Handed subclass progression ────────────────────────────────────

fn intimidating_blow_def() -> CardDef {
    let mut d = fighter_sc(
        INTIMIDATING_BLOW,
        "Intimidating Blow",
        "two_handed",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::ApplyStatus {
                target: Target::SingleEnemy,
                status: StatusType::Frightened,
                stacks: 1,
            },
        ],
        "Deal 10 damage. Apply 1 Frightened.",
    );
    d.milestone = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn reckless_attack_def() -> CardDef {
    let mut d = fighter_sc(
        RECKLESS_ATTACK,
        "Reckless Attack",
        "two_handed",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 15,
            },
            Effect::LoseHp { amount: 3 },
        ],
        "Deal 15 damage. Lose 3 HP.",
    );
    d.milestone = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn sundering_blow_def() -> CardDef {
    let mut d = fighter_sc(
        SUNDERING_BLOW,
        "Sundering Blow",
        "two_handed",
        1,
        vec![
            Effect::RemoveEnemyBlock {
                target: Target::SingleEnemy,
                amount: 6,
            },
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
        ],
        "Remove up to 6 enemy block. Deal 10 damage.",
    );
    d.milestone = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn reaving_strike_def() -> CardDef {
    let mut d = fighter_sc(
        REAVING_STRIKE,
        "Reaving Strike",
        "two_handed",
        2,
        vec![Effect::DealDamage {
            target: Target::AllEnemies,
            amount: 15,
        }],
        "Deal 15 damage to ALL enemies. Exhaust.",
    );
    d.exhaust = true;
    d.milestone = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Two-Handed subclass: Crusher synergy group ──────────────────────

fn brutal_swing_def() -> CardDef {
    let mut d = fighter_sc(BRUTAL_SWING, "Brutal Swing", "two_handed", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 10, bonus: 6 },
    ], "Deal 8 damage. If target has a debuff, deal 14 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

fn savage_presence_def() -> CardDef {
    let mut d = fighter_sc(SAVAGE_PRESENCE, "Savage Presence", "two_handed", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::SavageBlows, stacks: 2 },
    ], "Gain 2 Savage Blows. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

fn demolish_def() -> CardDef {
    let mut d = fighter_sc(DEMOLISH, "Demolish", "two_handed", 2, vec![
        Effect::RemoveEnemyBlock { target: Target::SingleEnemy, amount: 99 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 14 },
    ], "Remove ALL enemy block. Deal 14 damage.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn wrath_of_the_giant_def() -> CardDef {
    let mut d = fighter_sc(WRATH_OF_THE_GIANT, "Wrath of the Giant", "two_handed", 1, vec![
        Effect::DealDamagePerDebuffOnTarget { target: Target::SingleEnemy, base: 10, per_debuff: 5 },
    ], "Deal 6 damage +5 for each debuff on the target.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn executioners_blow_def() -> CardDef {
    let mut d = fighter_sc(EXECUTIONERS_BLOW, "Executioner's Blow", "two_handed", 2, vec![
        Effect::DealDamageScaledByMissingHpEnemy { target: Target::SingleEnemy, base: 12, per_10_percent_missing: 3 },
    ], "Deal 12 damage +3 per 10% HP the target is missing. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Two-Handed subclass: Berserker synergy group ─────────────────────

fn blood_frenzy_def() -> CardDef {
    let mut d = fighter_sc(BLOOD_FRENZY, "Blood Frenzy", "two_handed", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::LoseHp { amount: 3 },
        Effect::DrawCards { count: 2 },
    ], "Deal 10 damage. Lose 3 HP. Draw 2 cards.");
    d.tags = vec![CardTag::Attack];
    d
}

fn raging_blow_def() -> CardDef {
    let mut d = fighter_sc(RAGING_BLOW, "Raging Blow", "two_handed", 1, vec![
        Effect::DealDamageScaledByMissingHp { target: Target::SingleEnemy, base: 10, per_5_missing: 3, cap: 24 },
    ], "Deal 10 damage +3 per 5 missing HP (max +24).");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn berserk_roar_def() -> CardDef {
    let mut d = fighter_sc(BERSERK_ROAR, "Berserk Roar", "two_handed", 0, vec![
        Effect::LoseHp { amount: 3 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 2 },
    ], "Lose 3 HP. Apply 2 Threatened to ALL enemies. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn unleash_fury_def() -> CardDef {
    let mut d = fighter_sc(UNLEASH_FURY, "Unleash Fury", "two_handed", 0, vec![
        Effect::LoseHp { amount: 3 },
        Effect::GainEnergy { amount: 2 },
        Effect::DrawCards { count: 1 },
    ], "Lose 3 HP. Gain 2 energy. Draw 1 card. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn deathwish_def() -> CardDef {
    let mut d = fighter_sc(DEATHWISH, "Deathwish", "two_handed", 1, vec![
        Effect::DealDamageScaledByMissingHp { target: Target::AllEnemies, base: 10, per_5_missing: 2, cap: 16 },
    ], "Deal 10 damage +2 per 5 missing HP (max +16) to ALL enemies. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Dueling subclass progression ───────────────────────────────────────

fn sizing_up_def() -> CardDef {
    let mut d = fighter_sc(
        SIZING_UP,
        "Sizing Up",
        "dueling",
        0,
        vec![Effect::DrawCards { count: 2 }],
        "Draw 2 cards. Exhaust.",
    );
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn calculated_strike_def() -> CardDef {
    let mut d = fighter_sc(
        CALCULATED_STRIKE,
        "Calculated Strike",
        "dueling",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::DrawPerEnemyWithStatus {
                status: StatusType::Marked,
            },
        ],
        "Deal 10 damage. Draw 1 card per enemy with Marked.",
    );
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn precise_cut_def() -> CardDef {
    let mut d = fighter_sc(
        PRECISE_CUT,
        "Precise Cut",
        "dueling",
        1,
        vec![Effect::DealDamageScaledByMarked {
            target: Target::SingleEnemy,
            base_damage: 8,
            per_stack: 3,
        }],
        "Deal 8 + 3 per Marked stack damage.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn read_and_react_def() -> CardDef {
    let mut d = fighter_sc(
        READ_AND_REACT,
        "Read and React",
        "dueling",
        0,
        vec![
            Effect::DrawCards { count: 1 },
            Effect::GainBlock { amount: 5 },
            Effect::GainEnergy { amount: 1 },
        ],
        "Draw 1 card. Gain 5 block. Gain 1 energy. Exhaust.",
    );
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

fn expose_weakness_def() -> CardDef {
    let mut d = fighter_sc(
        EXPOSE_WEAKNESS,
        "Expose Weakness",
        "dueling",
        0,
        vec![Effect::DoubleStatus {
            target: Target::SingleEnemy,
            status: StatusType::Marked,
        }],
        "Double Marked stacks on target. Exhaust.",
    );
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn flurry_of_cuts_def() -> CardDef {
    let mut d = fighter_sc(
        FLURRY_OF_CUTS,
        "Flurry of Cuts",
        "dueling",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 4,
            },
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 4,
            },
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 4,
            },
            Effect::DrawCards { count: 1 },
        ],
        "Deal 4 damage 3 times. Draw 1 card.",
    );
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

// Placeholder definitions for cards whose engine mechanics aren't built yet.
// These use existing effects as stand-ins so they compile and appear in the
// reward pool / CARD_VOCAB. They will be reworked in later phases.

fn feint_def() -> CardDef {
    let mut d = fighter_sc(
        FEINT,
        "Feint",
        "dueling",
        0,
        vec![
            Effect::DrawAndDiscard { draw: 3, discard: 1 },
            Effect::GainBlock { amount: 5 },
        ],
        "Draw 3 cards, then discard 1. Gain 5 block.",
    );
    d.tags = vec![CardTag::Skill];
    d
}

fn patient_strike_def() -> CardDef {
    // Phase 2: will become retain + cost-reduction-on-retain.
    // Placeholder: deal 8 damage (no retain).
    let mut d = fighter_sc(
        PATIENT_STRIKE,
        "Patient Strike",
        "dueling",
        1,
        vec![Effect::DealDamage {
            target: Target::SingleEnemy,
            amount: 8,
        }],
        "Deal 8 damage. (Placeholder — will retain and cost 0 next turn.)",
    );
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn perfect_read_def() -> CardDef {
    // Phase 3: will become DrawToHandSize { target_size: 10 } + GainEnergy.
    // Placeholder: draw 5 cards, gain 1 energy, exhaust.
    let mut d = fighter_sc(
        PERFECT_READ,
        "Perfect Read",
        "dueling",
        0,
        vec![
            Effect::DrawCards { count: 5 },
            Effect::GainEnergy { amount: 1 },
        ],
        "Draw 5 cards. Gain 1 energy. Exhaust. (Placeholder — will draw to 10 cards.)",
    );
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn riposte_def() -> CardDef {
    let mut d = fighter_sc(
        RIPOSTE,
        "Riposte",
        "dueling",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::GainBlock { amount: 8 },
            Effect::ApplyStatus {
                target: Target::SingleEnemy,
                status: StatusType::Marked,
                stacks: 1,
            },
        ],
        "Deal 10 damage. Gain 8 block. Apply 1 Marked.",
    );
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn perfect_rhythm_def() -> CardDef {
    let mut d = fighter_sc(
        PERFECT_RHYTHM,
        "Perfect Rhythm",
        "dueling",
        1,
        vec![
            Effect::DrawCards { count: 4 },
            Effect::GainBlock { amount: 8 },
        ],
        "Draw 4 cards. Gain 8 block. Exhaust.",
    );
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Shared reward card definitions ────────────────────────────────────

fn power_strike_def() -> CardDef {
    let mut d = universal(
        POWER_STRIKE,
        "Power Strike",
        2,
        vec![Effect::DealDamage {
            target: Target::SingleEnemy,
            amount: 20,
        }],
        "Deal 20 damage.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn quick_slash_def() -> CardDef {
    let mut d = universal(
        QUICK_SLASH,
        "Quick Slash",
        0,
        vec![Effect::DealDamage {
            target: Target::SingleEnemy,
            amount: 4,
        }],
        "Deal 4 damage.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn vicious_strike_def() -> CardDef {
    let mut d = universal(
        VICIOUS_STRIKE,
        "Vicious Strike",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 8,
            },
            Effect::ApplyStatus {
                target: Target::SingleEnemy,
                status: StatusType::Bleeding,
                stacks: 1,
            },
        ],
        "Deal 8 damage. Apply 1 Bleeding.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn double_strike_def() -> CardDef {
    let mut d = universal(
        DOUBLE_STRIKE,
        "Double Strike",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 6,
            },
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 6,
            },
        ],
        "Deal 6 damage twice.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn brace_for_impact_def() -> CardDef {
    let mut d = universal(
        BRACE_FOR_IMPACT,
        "Brace for Impact",
        1,
        vec![Effect::GainBlock { amount: 12 }],
        "Gain 12 block.",
    );
    d.tags = vec![CardTag::Defense];
    d
}

fn hold_the_line_def() -> CardDef {
    let mut d = universal(
        HOLD_THE_LINE,
        "Hold the Line",
        1,
        vec![
            Effect::GainBlock { amount: 10 },
            Effect::ApplyStatus {
                target: Target::SingleEnemy,
                status: StatusType::Weakened,
                stacks: 1,
            },
        ],
        "Gain 10 block. Apply 1 Weakened.",
    );
    d.tags = vec![CardTag::Defense];
    d
}

fn taunt_def() -> CardDef {
    let mut d = universal(
        TAUNT,
        "Taunt",
        1,
        vec![Effect::ApplyStatus {
            target: Target::SingleEnemy,
            status: StatusType::Threatened,
            stacks: 2,
        }],
        "Apply 2 Threatened.",
    );
    d.tags = vec![CardTag::Skill];
    d
}

fn focus_def() -> CardDef {
    let mut d = universal(
        FOCUS,
        "Focus",
        0,
        vec![
            Effect::DrawCards { count: 1 },
            Effect::GainBlock { amount: 4 },
        ],
        "Draw 1 card. Gain 4 block.",
    );
    d.tags = vec![CardTag::Skill];
    d
}

fn martial_ascendancy_def() -> CardDef {
    let mut d = universal(
        MARTIAL_ASCENDANCY,
        "Martial Ascendancy",
        0,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 40,
            },
            Effect::DrawCards { count: 2 },
            Effect::GainEnergy { amount: 2 },
            Effect::GainBlock { amount: 20 },
        ],
        "Deal 40 damage. Draw 2 cards. Gain 2 energy. Gain 20 block.",
    );
    d.rarity = Rarity::Legendary;
    d.innate = true;
    d.tags = vec![CardTag::Attack, CardTag::Skill];
    d
}

// ── Legendary capstones ────────────────────────────────────────────────

fn iron_fortress_def() -> CardDef {
    let mut d = fighter_sc(
        IRON_FORTRESS,
        "Iron Fortress",
        "defense",
        1,
        vec![
            Effect::GainBlock { amount: 30 },
            Effect::RetainBlockPartial { amount: 10 },
        ],
        "Gain 30 block. Retain up to 10 block permanently. Exhaust.",
    );
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.milestone = true;
    d.tags = vec![CardTag::Power];
    d
}

fn titans_fury_def() -> CardDef {
    let mut d = fighter_sc(
        TITANS_FURY,
        "Titan's Fury",
        "two_handed",
        0,
        vec![
            Effect::ApplyStatus {
                target: Target::Player,
                status: StatusType::Empowered,
                stacks: 5,
            },
            Effect::DrawCards { count: 3 },
        ],
        "Gain 5 Empowered. Draw 3 cards. Exhaust.",
    );
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.milestone = true;
    d.tags = vec![CardTag::Power];
    d
}

fn coup_de_grace_def() -> CardDef {
    let mut d = fighter_sc(
        COUP_DE_GRACE,
        "Coup de Grace",
        "dueling",
        2,
        vec![Effect::DealDamageScaledByMarked {
            target: Target::SingleEnemy,
            base_damage: 10,
            per_stack: 3,
        }],
        "Deal 10 + 3 per Marked stack damage. Exhaust.",
    );
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.milestone = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── New Phase 2 cards ──────────────────────────────────────────────────

fn adrenaline_rush_def() -> CardDef {
    let mut d = fighter(
        ADRENALINE_RUSH,
        "Adrenaline Rush",
        0,
        vec![
            Effect::DrawCards { count: 2 },
            Effect::GainEnergy { amount: 1 },
        ],
        "Draw 2 cards. Gain 1 energy. Exhaust.",
    );
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn exploit_opening_def() -> CardDef {
    let mut d = fighter_sc(
        EXPLOIT_OPENING,
        "Exploit Opening",
        "dueling",
        0,
        vec![Effect::DealDamageScaledByMarked {
            target: Target::SingleEnemy,
            base_damage: 4,
            per_stack: 3,
        }],
        "Deal 4 + 3 per Marked stack damage.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

fn momentum_def() -> CardDef {
    let mut d = fighter_sc(
        MOMENTUM,
        "Momentum",
        "dueling",
        0,
        vec![Effect::DealDamagePerCardPlayed {
            target: Target::SingleEnemy,
            per_card: 3,
        }],
        "Deal 3 damage for each card played this turn.",
    );
    d.tags = vec![CardTag::Attack];
    d
}

// ── Race cards ─────────────────────────────────────────────────────────

fn improvise_def() -> CardDef {
    let mut d = universal(
        IMPROVISE,
        "Improvise",
        1,
        vec![Effect::DrawAndDiscard { draw: 3, discard: 1 }],
        "Draw 3 cards. Discard 1. Exhaust.",
    );
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d.inherent = true;
    d
}

fn fae_ancestry_def() -> CardDef {
    let mut d = universal(
        FAE_ANCESTRY,
        "Fae Ancestry",
        0,
        vec![
            Effect::GainBlock { amount: 8 },
            Effect::ApplyStatus {
                target: Target::Player,
                status: StatusType::Armored,
                stacks: 1,
            },
        ],
        "Gain 8 block. Gain 1 Armored. Innate. Exhaust.",
    );
    d.innate = true;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense];
    d.inherent = true;
    d
}

fn blood_price_def() -> CardDef {
    let mut d = universal(
        BLOOD_PRICE,
        "Blood Price",
        0,
        vec![
            Effect::GainEnergy { amount: 2 },
            Effect::LoseHp { amount: 4 },
        ],
        "Gain 2 energy. Lose 4 HP. Exhaust.",
    );
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d.inherent = true;
    d
}

fn stonewall_def() -> CardDef {
    let mut d = universal(
        STONEWALL,
        "Stonewall",
        1,
        vec![Effect::GainBlock { amount: 14 }],
        "Gain 14 block.",
    );
    d.tags = vec![CardTag::Defense];
    d.inherent = true;
    d
}

fn flash_bang_def() -> CardDef {
    let mut d = universal(
        FLASH_BANG,
        "Flash Bang",
        1,
        vec![Effect::ApplyStatus {
            target: Target::AllEnemies,
            status: StatusType::Weakened,
            stacks: 1,
        }],
        "Apply 1 Weakened to ALL enemies. Innate. Exhaust.",
    );
    d.innate = true;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d.inherent = true;
    d
}

fn nimble_dodge_def() -> CardDef {
    let mut d = universal(
        NIMBLE_DODGE,
        "Nimble Dodge",
        0,
        vec![
            Effect::GainBlock { amount: 4 },
            Effect::DrawCards { count: 1 },
        ],
        "Gain 4 block. Draw 1 card.",
    );
    d.tags = vec![CardTag::Defense];
    d.inherent = true;
    d
}

fn savage_charge_def() -> CardDef {
    let mut d = universal(
        SAVAGE_CHARGE,
        "Savage Charge",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 18,
            },
            Effect::LoseHp { amount: 4 },
        ],
        "Deal 18 damage. Lose 4 HP.",
    );
    d.tags = vec![CardTag::Attack];
    d.inherent = true;
    d
}

fn shiv_def() -> CardDef {
    let mut d = universal(
        SHIV,
        "Shiv",
        0,
        vec![Effect::DealDamage {
            target: Target::SingleEnemy,
            amount: 4,
        }],
        "Deal 4 damage. Recycle.",
    );
    d.recycle = true;
    d.tags = vec![CardTag::Attack];
    d.inherent = true;
    d
}

fn dragon_breath_def() -> CardDef {
    let mut d = universal(
        DRAGON_BREATH,
        "Dragon Breath",
        2,
        vec![Effect::DealDamage {
            target: Target::AllEnemies,
            amount: 12,
        }],
        "Deal 12 damage to ALL enemies. Exhaust.",
    );
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d.inherent = true;
    d
}

fn pounce_def() -> CardDef {
    let mut d = universal(
        POUNCE,
        "Pounce",
        1,
        vec![
            Effect::DealDamage {
                target: Target::SingleEnemy,
                amount: 10,
            },
            Effect::DrawCards { count: 1 },
        ],
        "Deal 10 damage. Draw 1 card.",
    );
    d.tags = vec![CardTag::Attack];
    d.inherent = true;
    d
}

/// Map a race name to its starter card ID.
pub fn race_card_id(race: &str) -> &'static str {
    match race {
        "human" => IMPROVISE,
        "high_elf" => FAE_ANCESTRY,
        "dark_elf" => BLOOD_PRICE,
        "dwarf" => STONEWALL,
        "gnome" => FLASH_BANG,
        "halfling" => NIMBLE_DODGE,
        "orc" => SAVAGE_CHARGE,
        "goblin" => SHIV,
        "dragonkin" => DRAGON_BREATH,
        "pantheran" => POUNCE,
        _ => panic!("unknown race: {}", race),
    }
}

// ── Background cards ──────────────────────────────────────────────────

fn background_card(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.to_string(),
        name: name.to_string(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.to_string(),
        class: None,
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: true,
    }
}

fn battle_discipline_def() -> CardDef {
    background_card(BATTLE_DISCIPLINE, "Battle Discipline", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::GainBlock { amount: 4 },
    ], "Deal 8 damage. Gain 4 block.")
}

fn studied_analysis_def() -> CardDef {
    background_card(STUDIED_ANALYSIS, "Studied Analysis", 1, vec![
        Effect::DrawCards { count: 2 },
        Effect::GainBlock { amount: 3 },
    ], "Draw 2 cards. Gain 3 block.")
}

fn minor_blessing_def() -> CardDef {
    background_card(MINOR_BLESSING, "Minor Blessing", 0, vec![
        Effect::GainBlock { amount: 6 },
        Effect::Heal { amount: 2 },
    ], "Gain 6 block. Heal 2 HP.")
}

fn dirty_trick_def() -> CardDef {
    background_card(DIRTY_TRICK, "Dirty Trick", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
    ], "Deal 4 damage. Apply 1 Weakened.")
}

fn commanding_presence_def() -> CardDef {
    background_card(COMMANDING_PRESENCE, "Commanding Presence", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::RandomEnemy, status: StatusType::Threatened, stacks: 1 },
    ], "Gain 10 block. Apply 1 Threatened to a random enemy.")
}

fn survival_instinct_def() -> CardDef {
    background_card(SURVIVAL_INSTINCT, "Survival Instinct", 0, vec![
        Effect::GainBlock { amount: 5 },
        Effect::DrawCards { count: 1 },
    ], "Gain 5 block. Draw 1 card.")
}

fn improvised_weapon_def() -> CardDef {
    background_card(IMPROVISED_WEAPON, "Improvised Weapon", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 12 },
    ], "Deal 12 damage.")
}

fn sea_legs_def() -> CardDef {
    background_card(SEA_LEGS, "Sea Legs", 1, vec![
        Effect::GainBlock { amount: 12 },
    ], "Gain 12 block.")
}

fn backstab_def() -> CardDef {
    background_card(BACKSTAB, "Backstab", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 6 },
    ], "Deal 6 damage.")
}

fn inner_focus_def() -> CardDef {
    background_card(INNER_FOCUS, "Inner Focus", 1, vec![
        Effect::DrawCards { count: 3 },
    ], "Draw 3 cards.")
}

fn resourceful_def() -> CardDef {
    background_card(RESOURCEFUL, "Resourceful", 0, vec![
        Effect::GainEnergy { amount: 1 },
        Effect::GainBlock { amount: 3 },
    ], "Gain 1 energy. Gain 3 block.")
}

fn dazzle_def() -> CardDef {
    background_card(DAZZLE, "Dazzle", 0, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
    ], "Apply 1 Weakened to ALL enemies.")
}

// ══════════════════════════════════════════════════════════════════════
// BARBARIAN CLASS CARDS
// ══════════════════════════════════════════════════════════════════════

// ── Barbarian class card ─────────────────────────────────────────────

fn bloodlust_def() -> CardDef {
    let mut d = barbarian(BLOODLUST, "Bloodlust", 1, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 3 },
        Effect::GainBlock { amount: 6 },
    ], "Gain 3 Rage. Gain 6 block. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Barbarian themed basics ──────────────────────────────────────────

fn raging_strike_def() -> CardDef {
    let mut d = barbarian_sc(RAGING_STRIKE, "Raging Strike", "berserker", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Deal 10 damage. Gain 1 Rage.");
    d.tags = vec![CardTag::Attack];
    d
}

fn totem_guard_def() -> CardDef {
    let mut d = barbarian_sc(TOTEM_GUARD, "Totem Guard", "totem_warrior", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Gain 10 block. Gain 1 Rage.");
    d.tags = vec![CardTag::Defense];
    d
}

fn frenzied_slash_def() -> CardDef {
    let mut d = barbarian_sc(FRENZIED_SLASH, "Frenzied Slash", "frenzy", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Deal 8 damage. Draw 1 card.");
    d.tags = vec![CardTag::Attack];
    d
}

// ── Barbarian shared auto-grants ─────────────────────────────────────

fn battle_fury_def() -> CardDef {
    let mut d = barbarian(BATTLE_FURY, "Battle Fury", 0, vec![
        Effect::GainEnergy { amount: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 2 },
    ], "Gain 1 energy. Gain 2 Rage. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn thick_skin_def() -> CardDef {
    let mut d = barbarian(THICK_SKIN, "Thick Skin", 0, vec![
        Effect::GainBlock { amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Gain 8 block. Draw 1 card. Innate. Exhaust.");
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Defense];
    d
}

fn primal_surge_def() -> CardDef {
    let mut d = barbarian(PRIMAL_SURGE, "Primal Surge", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::GainEnergy { amount: 1 },
    ], "Draw 2 cards. Gain 1 energy. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn undying_rage_def() -> CardDef {
    let mut d = barbarian(UNDYING_RAGE, "Undying Rage", 0, vec![
        Effect::BlockEqualMissingHp { max: 20 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Gain block equal to missing HP (max 20). Gain 1 Rage. Innate. Exhaust.");
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Defense];
    d
}

// ── Berserker: Bloodrage synergy ─────────────────────────────────────

fn blood_offering_def() -> CardDef {
    let mut d = barbarian_sc(BLOOD_OFFERING, "Blood Offering", "berserker", 0, vec![
        Effect::LoseHp { amount: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 3 },
        Effect::GainBlock { amount: 5 },
    ], "Lose 3 HP. Gain 3 Rage. Gain 5 block.");
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

fn fury_unleashed_def() -> CardDef {
    let mut d = barbarian_sc(FURY_UNLEASHED, "Fury Unleashed", "berserker", 1, vec![
        Effect::DealDamagePerPlayerStatus { target: Target::SingleEnemy, status: StatusType::Rage, per_stack: 4 },
    ], "Deal 4 damage per Rage stack.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn bloodbath_def() -> CardDef {
    let mut d = barbarian_sc(BLOODBATH, "Bloodbath", "berserker", 1, vec![
        Effect::LoseHp { amount: 5 },
        Effect::DealDamage { target: Target::AllEnemies, amount: 15 },
    ], "Lose 5 HP. Deal 15 damage to ALL enemies.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn pain_is_power_def() -> CardDef {
    let mut d = barbarian_sc(PAIN_IS_POWER, "Pain is Power", "berserker", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::PainIsPower, stacks: 1 },
    ], "Whenever you lose HP, gain that much Rage. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

fn berserker_deathwish_def() -> CardDef {
    let mut d = barbarian_sc(BERSERKER_DEATHWISH, "Deathwish", "berserker", 1, vec![
        Effect::LoseHp { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 4 },
        Effect::DealDamagePerPlayerStatus { target: Target::SingleEnemy, status: StatusType::Rage, per_stack: 4 },
    ], "Lose 8 HP. Gain 4 Rage. Deal 4 damage per Rage stack. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Berserker: Rampage synergy ───────────────────────────────────────

fn wild_swing_def() -> CardDef {
    let mut d = barbarian_sc(WILD_SWING, "Wild Swing", "berserker", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Deal 4 damage. Gain 1 Rage.");
    d.tags = vec![CardTag::Attack];
    d
}

fn berserk_flurry_def() -> CardDef {
    let mut d = barbarian_sc(BERSERK_FLURRY, "Berserk Flurry", "berserker", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DrawCards { count: 1 },
    ], "Deal 4 damage 3 times. Draw 1 card.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn savage_momentum_def() -> CardDef {
    let mut d = barbarian_sc(SAVAGE_MOMENTUM, "Savage Momentum", "berserker", 1, vec![
        Effect::DealDamagePerCardPlayed { target: Target::SingleEnemy, per_card: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Deal 3 damage per card played this turn. Gain 1 Rage.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn unstoppable_def() -> CardDef {
    let mut d = barbarian_sc(UNSTOPPABLE, "Unstoppable", "berserker", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 6 },
        Effect::GainBlock { amount: 6 },
        Effect::DrawCards { count: 1 },
        Effect::GainEnergy { amount: 1 },
    ], "Deal 6 damage. Gain 6 block. Draw 1 card. Gain 1 energy. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack, CardTag::Skill];
    d
}

fn rampage_def() -> CardDef {
    let mut d = barbarian_sc(RAMPAGE, "Rampage", "berserker", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 2 },
    ], "Deal 5 damage 4 times. Gain 2 Rage. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Totem Warrior: Spirit Shield synergy ─────────────────────────────

fn spirit_ward_def() -> CardDef {
    let mut d = barbarian_sc(SPIRIT_WARD, "Spirit Ward", "totem_warrior", 1, vec![
        Effect::LoseHp { amount: 3 },
        Effect::GainBlock { amount: 15 },
    ], "Lose 3 HP. Gain 15 block.");
    d.tags = vec![CardTag::Defense];
    d
}

fn ancestral_shield_def() -> CardDef {
    let mut d = barbarian_sc(ANCESTRAL_SHIELD, "Ancestral Shield", "totem_warrior", 1, vec![
        Effect::LoseHp { amount: 4 },
        Effect::GainBlock { amount: 14 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 3 },
    ], "Lose 4 HP. Gain 14 block. Gain 3 Rage.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense];
    d
}

fn warding_totem_card_def() -> CardDef {
    let mut d = barbarian_sc(WARDING_TOTEM_CARD, "Warding Totem", "totem_warrior", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::WardingTotem, stacks: 1 },
    ], "Whenever you lose HP, gain 3 block. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

fn fortified_rage_def() -> CardDef {
    let mut d = barbarian_sc(FORTIFIED_RAGE, "Fortified Rage", "totem_warrior", 1, vec![
        Effect::GainBlockPerStatusStack { status: StatusType::Rage, multiplier: 2 },
    ], "Gain block equal to 2x your Rage stacks.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense];
    d
}

fn unbreaking_spirit_def() -> CardDef {
    let mut d = barbarian_sc(UNBREAKING_SPIRIT, "Unbreaking Spirit", "totem_warrior", 1, vec![
        Effect::LoseHp { amount: 6 },
        Effect::GainBlock { amount: 25 },
        Effect::BlockFloor { amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 3 },
    ], "Lose 6 HP. Gain 25 block. Block cannot go below 5. Gain 3 Rage. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

// ── Totem Warrior: Ancestral synergy ─────────────────────────────────

fn war_cry_def() -> CardDef {
    let mut d = barbarian_sc(WAR_CRY, "War Cry", "totem_warrior", 1, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 3 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
    ], "Gain 3 Rage. Apply 1 Weakened to ALL enemies.");
    d.tags = vec![CardTag::Skill];
    d
}

fn spirit_mend_def() -> CardDef {
    let mut d = barbarian_sc(SPIRIT_MEND, "Spirit Mend", "totem_warrior", 1, vec![
        Effect::Heal { amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Heal 5 HP. Gain 3 Mending. Gain 1 Rage.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn vengeful_ancestors_def() -> CardDef {
    let mut d = barbarian_sc(VENGEFUL_ANCESTORS, "Vengeful Ancestors", "totem_warrior", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::Heal { amount: 4 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 2 },
    ], "Deal 8 damage. Heal 4 HP. Gain 2 Rage.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn totem_of_renewal_def() -> CardDef {
    let mut d = barbarian_sc(TOTEM_OF_RENEWAL, "Totem of Renewal", "totem_warrior", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 2 },
    ], "Gain 2 Mending. Gain 2 Rage. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

fn ancestors_embrace_def() -> CardDef {
    let mut d = barbarian_sc(ANCESTORS_EMBRACE, "Ancestors' Embrace", "totem_warrior", 1, vec![
        Effect::Heal { amount: 10 },
        Effect::GainBlock { amount: 15 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 3 },
    ], "Heal 10 HP. Gain 15 block. Gain 3 Mending. Gain 3 Rage. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

// ── Frenzy: Adrenaline synergy ───────────────────────────────────────

fn desperate_strike_def() -> CardDef {
    let mut d = barbarian_sc(DESPERATE_STRIKE, "Desperate Strike", "frenzy", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::DrawCards { count: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Deal 8 damage. Draw 1 card. Gain 1 Rage.");
    d.tags = vec![CardTag::Attack];
    d
}

fn adrenaline_spike_def() -> CardDef {
    let mut d = barbarian_sc(ADRENALINE_SPIKE, "Adrenaline Spike", "frenzy", 0, vec![
        Effect::LoseHp { amount: 3 },
        Effect::DrawCards { count: 2 },
        Effect::GainEnergy { amount: 1 },
    ], "Lose 3 HP. Draw 2 cards. Gain 1 energy. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn feral_instinct_def() -> CardDef {
    let mut d = barbarian_sc(FERAL_INSTINCT, "Feral Instinct", "frenzy", 0, vec![
        Effect::DrawCards { count: 1 },
        Effect::GainBlock { amount: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Draw 1 card. Gain 3 block. Gain 1 Rage. Recycle.");
    d.recycle = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn berserkers_trance_card_def() -> CardDef {
    let mut d = barbarian_sc(BERSERKERS_TRANCE_CARD, "Berserker's Trance", "frenzy", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::BerserkersTrance, stacks: 1 },
    ], "At the start of each turn, lose 2 HP and draw 1 extra card. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Power];
    d
}

fn deaths_door_def() -> CardDef {
    let mut d = barbarian_sc(DEATHS_DOOR, "Death's Door", "frenzy", 0, vec![
        Effect::LoseHp { amount: 10 },
        Effect::DrawCards { count: 5 },
        Effect::GainEnergy { amount: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 5 },
    ], "Lose 10 HP. Draw 5 cards. Gain 2 energy. Gain 5 Rage. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Frenzy: Overwhelm synergy ────────────────────────────────────────

fn flailing_strike_def() -> CardDef {
    let mut d = barbarian_sc(FLAILING_STRIKE, "Flailing Strike", "frenzy", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DealDamage { target: Target::RandomEnemy, amount: 4 },
    ], "Deal 4 damage twice. Deal 4 damage to a random enemy.");
    d.tags = vec![CardTag::Attack];
    d
}

fn frenzy_card_def() -> CardDef {
    let mut d = barbarian_sc(FRENZY_CARD, "Frenzy", "frenzy", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 3 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 3 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 3 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 1 },
    ], "Deal 3 damage 4 times. Gain 1 Rage.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn whirlwind_fury_def() -> CardDef {
    let mut d = barbarian_sc(WHIRLWIND_FURY, "Whirlwind Fury", "frenzy", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 5 },
        Effect::DealDamage { target: Target::AllEnemies, amount: 5 },
        Effect::DealDamage { target: Target::AllEnemies, amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Rage, stacks: 2 },
    ], "Deal 5 damage to ALL enemies 3 times. Gain 2 Rage.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn one_thousand_cuts_def() -> CardDef {
    let mut d = barbarian_sc(ONE_THOUSAND_CUTS, "One Thousand Cuts", "frenzy", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 2 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 2 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 2 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 2 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 2 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 2 },
        Effect::DrawCards { count: 1 },
    ], "Deal 2 damage 6 times. Draw 1 card. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ══════════════════════════════════════════════════════════════════════
// MONK CLASS CARDS
// ══════════════════════════════════════════════════════════════════════

fn inner_peace_def() -> CardDef {
    let mut d = monk(INNER_PEACE, "Inner Peace", 1, vec![
        Effect::Heal { amount: 12 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 2 },
    ], "Heal 12 HP. Gain 2 Ki.");
    d.card_type = CardType::Consumable;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Monk shared auto-grants ──────────────────────────────────────────

fn centered_strike_def() -> CardDef {
    let mut d = monk(CENTERED_STRIKE, "Centered Strike", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Deal 5 damage. Gain 1 Ki. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn swift_footwork_def() -> CardDef {
    let mut d = monk(SWIFT_FOOTWORK, "Swift Footwork", 0, vec![
        Effect::GainBlock { amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Gain 8 block. Draw 1 card. Innate. Exhaust.");
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Defense];
    d
}

fn ki_surge_def() -> CardDef {
    let mut d = monk(KI_SURGE, "Ki Surge", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 2 },
    ], "Draw 2 cards. Gain 2 Ki. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn meditation_def() -> CardDef {
    let mut d = monk(MEDITATION, "Meditation", 0, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Gain 10 block. Gain 1 Ki. Innate. Exhaust.");
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Defense];
    d
}

// ── Monk shared pool (level 1) ──────────────────────────────────────

fn ki_focus_def() -> CardDef {
    let mut d = monk(KI_FOCUS, "Ki Focus", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
        Effect::DrawCards { count: 1 },
    ], "Gain 1 Ki. Draw 1 card. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn ki_strike_def() -> CardDef {
    let mut d = monk(KI_STRIKE, "Ki Strike", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Deal 10 damage. Gain 1 Ki.");
    d.tags = vec![CardTag::Attack];
    d
}

fn ki_guard_def() -> CardDef {
    let mut d = monk(KI_GUARD, "Ki Guard", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Gain 10 block. Gain 1 Ki.");
    d.tags = vec![CardTag::Defense];
    d
}

// ── Open Hand: Flurry synergy ────────────────────────────────────────

fn rapid_strikes_def() -> CardDef {
    let mut d = monk_sc(RAPID_STRIKES, "Rapid Strikes", "open_hand", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Deal 5 damage twice. Gain 1 Ki.");
    d.tags = vec![CardTag::Attack];
    d
}

fn palm_strike_def() -> CardDef {
    let mut d = monk_sc(PALM_STRIKE, "Palm Strike", "open_hand", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 7 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Deal 7 damage. Gain 1 Ki.");
    d.tags = vec![CardTag::Attack];
    d
}

fn whirlwind_kick_def() -> CardDef {
    let mut d = monk_sc(WHIRLWIND_KICK, "Whirlwind Kick", "open_hand", 1, vec![
        Effect::DealDamagePerCardPlayed { target: Target::AllEnemies, per_card: 4 },
    ], "Deal 4 damage to ALL enemies for each card played this turn.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn hundred_fists_def() -> CardDef {
    let mut d = monk_sc(HUNDRED_FISTS, "Hundred Fists", "open_hand", 1, vec![
        Effect::DealDamagePerCardPlayed { target: Target::SingleEnemy, per_card: 4 },
    ], "Deal 4 damage for each card played this turn.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn endless_barrage_def() -> CardDef {
    let mut d = monk_sc(ENDLESS_BARRAGE, "Endless Barrage", "open_hand", 2, vec![
        Effect::DealDamagePerCardPlayed { target: Target::SingleEnemy, per_card: 4 },
        Effect::DrawCards { count: 2 },
    ], "Deal 4 damage per card played this turn. Draw 2 cards. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Open Hand: Ki Burst synergy ──────────────────────────────────────

fn focused_strike_def() -> CardDef {
    let mut d = monk_sc(FOCUSED_STRIKE, "Focused Strike", "open_hand", 1, vec![
        Effect::DealDamageSpendKi { target: Target::SingleEnemy, base: 8, ki_cost: 3, bonus: 16 },
    ], "Deal 8 damage. Spend 3 Ki: deal 16 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

fn ki_shield_def() -> CardDef {
    let mut d = monk_sc(KI_SHIELD, "Ki Shield", "open_hand", 1, vec![
        Effect::GainBlockSpendKi { base: 8, ki_cost: 3, bonus: 16 },
    ], "Gain 8 block. Spend 3 Ki: gain 16 instead.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense];
    d
}

fn ki_channeling_def() -> CardDef {
    let mut d = monk_sc(KI_CHANNELING, "Ki Channeling", "open_hand", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 3 },
        Effect::GainBlock { amount: 3 },
    ], "Gain 3 Ki. Gain 3 block. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn quivering_palm_def() -> CardDef {
    let mut d = monk_sc(QUIVERING_PALM, "Quivering Palm", "open_hand", 1, vec![
        Effect::GainBlockPerKiStack { multiplier: 2 },
        Effect::DealDamageSpendKi { target: Target::SingleEnemy, base: 10, ki_cost: 5, bonus: 30 },
    ], "Gain block equal to 2x Ki. Spend 5 Ki: deal 30 damage. Otherwise deal 10. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn transcendence_def() -> CardDef {
    let mut d = monk_sc(TRANSCENDENCE, "Transcendence", "open_hand", 0, vec![
        Effect::DrawCards { count: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 5 },
        Effect::DealDamagePerCardPlayed { target: Target::AllEnemies, per_card: 3 },
    ], "Draw 3 cards. Gain 5 Ki. Deal 3 AoE damage per card played. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Attack, CardTag::Skill];
    d
}

// ── Way of Shadow: Pressure Point synergy ────────────────────────────

fn nerve_strike_def() -> CardDef {
    let mut d = monk_sc(NERVE_STRIKE, "Nerve Strike", "way_of_shadow", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Deal 8 damage. Apply 1 Weakened. Gain 1 Ki.");
    d.tags = vec![CardTag::Attack];
    d
}

fn pressure_point_strike_def() -> CardDef {
    let mut d = monk_sc(PRESSURE_POINT_STRIKE, "Pressure Point Strike", "way_of_shadow", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 7, bonus: 7 },
    ], "Deal 7 damage. If target has a debuff, deal 14 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

fn dim_mak_def() -> CardDef {
    let mut d = monk_sc(DIM_MAK, "Dim Mak", "way_of_shadow", 1, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Threatened, stacks: 1 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
    ], "Apply 1 Weakened and 1 Threatened. Deal 5 damage.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn crippling_blow_def() -> CardDef {
    let mut d = monk_sc(CRIPPLING_BLOW, "Crippling Blow", "way_of_shadow", 1, vec![
        Effect::DealDamagePerDebuffOnTarget { target: Target::SingleEnemy, base: 4, per_debuff: 5 },
    ], "Deal 4 damage +5 per debuff type on target.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn weakening_aura_def() -> CardDef {
    let mut d = monk_sc(WEAKENING_AURA, "Weakening Aura", "way_of_shadow", 0, vec![
        Effect::ApplyStatusPerCardPlayed { target: Target::AllEnemies, status: StatusType::Weakened, per_card: 1 },
    ], "Apply Weakened to ALL enemies equal to cards played this turn. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn death_touch_def() -> CardDef {
    let mut d = monk_sc(DEATH_TOUCH, "Death Touch", "way_of_shadow", 2, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 2 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Threatened, stacks: 2 },
        Effect::DealDamagePerDebuffOnTarget { target: Target::SingleEnemy, base: 8, per_debuff: 8 },
    ], "Apply 2 Weakened and 2 Threatened. Deal 8 +8 per debuff type. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Way of Shadow: Evasion synergy ───────────────────────────────────

fn shadow_step_def() -> CardDef {
    let mut d = monk_sc(SHADOW_STEP, "Shadow Step", "way_of_shadow", 0, vec![
        Effect::GainBlock { amount: 5 },
        Effect::DrawCards { count: 1 },
    ], "Gain 5 block. Draw 1 card.");
    d.tags = vec![CardTag::Defense];
    d
}

fn deflecting_palm_def() -> CardDef {
    let mut d = monk_sc(DEFLECTING_PALM, "Deflecting Palm", "way_of_shadow", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 2 },
    ], "Gain 10 block. Gain 2 Ki.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense];
    d
}

fn counter_stance_def() -> CardDef {
    let mut d = monk_sc(COUNTER_STANCE, "Counter Stance", "way_of_shadow", 1, vec![
        Effect::GainBlockIfDamagedLastTurn { amount: 15 },
        Effect::DealDamageIfDamagedLastTurn { target: Target::SingleEnemy, amount: 10 },
    ], "If damaged last turn: gain 15 block, deal 10 damage.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Defense, CardTag::Attack];
    d
}

fn phantom_form_def() -> CardDef {
    let mut d = monk_sc(PHANTOM_FORM, "Phantom Form", "way_of_shadow", 1, vec![
        Effect::GainBlock { amount: 12 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 3 },
    ], "Gain 12 block. Gain 3 Armored. Gain 3 Ki. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

fn shadow_kill_def() -> CardDef {
    let mut d = monk_sc(SHADOW_KILL, "Shadow Kill", "way_of_shadow", 0, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 3 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 3 },
        Effect::DealDamagePerDebuffOnTarget { target: Target::SingleEnemy, base: 10, per_debuff: 10 },
    ], "Apply 3 Weakened + 3 Threatened to ALL. Deal 10 +10 per debuff type. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Attack, CardTag::Skill];
    d
}

// ── Iron Fist: Stance Flow synergy ───────────────────────────────────

fn stance_shift_def() -> CardDef {
    let mut d = monk_sc(STANCE_SHIFT, "Stance Shift", "iron_fist", 0, vec![
        Effect::EnterStance { stance: StatusType::StanceAggressive },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Enter Aggressive Stance. Gain 1 Ki.");
    d.tags = vec![CardTag::Skill];
    d
}

fn tigers_fury_def() -> CardDef {
    let mut d = monk_sc(TIGERS_FURY, "Tiger's Fury", "iron_fist", 1, vec![
        Effect::EnterStance { stance: StatusType::StanceAggressive },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 12 },
    ], "Enter Aggressive Stance. Deal 12 damage.");
    d.tags = vec![CardTag::Attack];
    d
}

fn cranes_wing_def() -> CardDef {
    let mut d = monk_sc(CRANES_WING, "Crane's Wing", "iron_fist", 1, vec![
        Effect::EnterStance { stance: StatusType::StanceDefensive },
        Effect::GainBlock { amount: 12 },
    ], "Enter Defensive Stance. Gain 12 block.");
    d.tags = vec![CardTag::Defense];
    d
}

fn flowing_water_def() -> CardDef {
    let mut d = monk_sc(FLOWING_WATER, "Flowing Water", "iron_fist", 0, vec![
        Effect::EnterStance { stance: StatusType::StanceFlowing },
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 1 },
    ], "Enter Flowing Stance. Draw 2 cards. Gain 1 Ki.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn monk_dragons_breath_def() -> CardDef {
    let mut d = monk_sc(MONK_DRAGONS_BREATH, "Dragon's Breath", "iron_fist", 1, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 12 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 2 },
        Effect::EnterStance { stance: StatusType::StanceAggressive },
    ], "Deal 12 AoE damage. Gain 2 Ki. Enter Aggressive Stance.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn perfect_harmony_def() -> CardDef {
    let mut d = monk_sc(PERFECT_HARMONY, "Perfect Harmony", "iron_fist", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 15 },
        Effect::GainBlock { amount: 15 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 3 },
    ], "Deal 15 damage. Gain 15 block. Gain 3 Ki. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

// ── Iron Fist: Iron Will synergy ─────────────────────────────────────

fn iron_skin_def() -> CardDef {
    let mut d = monk_sc(IRON_SKIN, "Iron Skin", "iron_fist", 1, vec![
        Effect::GainBlockSpendKi { base: 8, ki_cost: 2, bonus: 14 },
    ], "Gain 8 block. Spend 2 Ki: gain 14 instead.");
    d.tags = vec![CardTag::Defense];
    d
}

fn ki_barrier_def() -> CardDef {
    let mut d = monk_sc(KI_BARRIER, "Ki Barrier", "iron_fist", 0, vec![
        Effect::GainBlockPerKiStack { multiplier: 2 },
    ], "Gain block equal to 2x Ki stacks.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense];
    d
}

fn stone_body_def() -> CardDef {
    let mut d = monk_sc(STONE_BODY, "Stone Body", "iron_fist", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 2 },
    ], "Gain 10 block. Gain 2 Armored. Gain 2 Ki. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

fn diamond_soul_def() -> CardDef {
    let mut d = monk_sc(DIAMOND_SOUL, "Diamond Soul", "iron_fist", 2, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 5 },
        Effect::GainBlockPerKiStack { multiplier: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 3 },
    ], "Gain 5 Ki. Gain block equal to 3x Ki. Gain 3 Armored. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

fn fist_of_north_star_def() -> CardDef {
    let mut d = monk_sc(FIST_OF_NORTH_STAR, "Fist of the North Star", "iron_fist", 0, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 20 },
        Effect::GainBlock { amount: 20 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Ki, stacks: 5 },
        Effect::EnterStance { stance: StatusType::StanceFlowing },
    ], "Deal 20 damage. Gain 20 block. Gain 5 Ki. Enter Flowing Stance. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

// ══════════════════════════════════════════════════════════════════════
// WARLOCK CLASS CARDS
// ══════════════════════════════════════════════════════════════════════

fn warlock(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Warlock),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn warlock_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Warlock),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

// ── Warlock signature cards ─────────────────────────────────────────

fn eldritch_blast_def() -> CardDef {
    let mut d = warlock(ELDRITCH_BLAST, "Eldritch Blast", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
    ], "Deal 10 damage. Recycle.");
    d.recycle = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn hex_def() -> CardDef {
    let mut d = warlock(HEX, "Hex", 0, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Hexed, stacks: 1 },
    ], "Apply Hexed to an enemy. Concentration.");
    d.concentration = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Warlock shared cards ────────────────────────────────────────────

fn dark_bargain_def() -> CardDef {
    let mut d = warlock(DARK_BARGAIN, "Dark Bargain", 0, vec![
        Effect::LoseHp { amount: 3 },
        Effect::DrawCards { count: 2 },
        Effect::GainEnergy { amount: 1 },
    ], "Lose 3 HP. Draw 2 cards. Gain 1 energy. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn maledict_def() -> CardDef {
    let mut d = warlock(MALEDICT, "Maledict", 1, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Marked, stacks: 1 },
    ], "Apply 1 Weakened and 1 Marked to an enemy.");
    d.tags = vec![CardTag::Skill];
    d
}

fn pact_barrier_def() -> CardDef {
    let mut d = warlock(PACT_BARRIER, "Pact Barrier", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Gain 8 block. Draw 1 card.");
    d.tags = vec![CardTag::Defense];
    d
}

fn siphon_bolt_def() -> CardDef {
    let mut d = warlock(SIPHON_BOLT, "Siphon Bolt", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 7, bonus: 7 },
    ], "Deal 7 damage. If target has a debuff, deal 14 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

// ── Infernal: Infernal Burn synergy ─────────────────────────────────

fn hellbrand_def() -> CardDef {
    let mut d = warlock_sc(HELLBRAND, "Hellbrand", "infernal", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Bleeding, stacks: 2 },
    ], "Deal 8 damage. Apply 2 Bleeding.");
    d.tags = vec![CardTag::Attack];
    d
}

fn cinder_feast_def() -> CardDef {
    let mut d = warlock_sc(CINDER_FEAST, "Cinder Feast", "infernal", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::Heal { amount: 4 },
    ], "Deal 8 damage. Heal 4 HP.");
    d.tags = vec![CardTag::Attack];
    d
}

fn balefire_cataclysm_def() -> CardDef {
    let mut d = warlock_sc(BALEFIRE_CATACLYSM, "Balefire Cataclysm", "infernal", 2, vec![
        Effect::LoseHp { amount: 6 },
        Effect::DealDamage { target: Target::AllEnemies, amount: 18 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Bleeding, stacks: 2 },
    ], "Lose 6 HP. Deal 18 damage to ALL enemies. Apply 2 Bleeding to ALL. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Infernal: Infernal Sustain synergy ──────────────────────────────

fn blood_tithe_def() -> CardDef {
    let mut d = warlock_sc(BLOOD_TITHE, "Blood Tithe", "infernal", 0, vec![
        Effect::LoseHp { amount: 4 },
        Effect::Heal { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Empowered, stacks: 1 },
    ], "Lose 4 HP. Heal 8 HP. Gain 1 Empowered. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn feast_of_cinders_def() -> CardDef {
    let mut d = warlock_sc(FEAST_OF_CINDERS, "Feast of Cinders", "infernal", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 12 },
        Effect::Heal { amount: 8 },
    ], "Deal 12 damage. Heal 8 HP. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Hexblade: Hexblade Duel synergy ─────────────────────────────────

fn pact_blade_def() -> CardDef {
    let mut d = warlock_sc(PACT_BLADE, "Pact Blade", "hexblade", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Marked, stacks: 1 },
    ], "Deal 10 damage. Apply 1 Marked.");
    d.tags = vec![CardTag::Attack];
    d
}

fn soul_parry_def() -> CardDef {
    let mut d = warlock_sc(SOUL_PARRY, "Soul Parry", "hexblade", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::DealDamageIfDamagedLastTurn { target: Target::SingleEnemy, amount: 10 },
    ], "Gain 10 block. If you took damage last turn, deal 10 damage.");
    d.tags = vec![CardTag::Defense, CardTag::Attack];
    d
}

fn dusk_duel_def() -> CardDef {
    let mut d = warlock_sc(DUSK_DUEL, "Dusk Duel", "hexblade", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 18 },
        Effect::GainBlock { amount: 18 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Marked, stacks: 2 },
    ], "Deal 18 damage. Gain 18 block. Apply 2 Marked. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

// ── Hexblade: Hexblade Curse synergy ────────────────────────────────

fn brand_the_weak_def() -> CardDef {
    let mut d = warlock_sc(BRAND_THE_WEAK, "Brand the Weak", "hexblade", 1, vec![
        Effect::DealDamagePerDebuffOnTarget { target: Target::SingleEnemy, base: 4, per_debuff: 5 },
    ], "Deal 4 damage + 5 per debuff type on target.");
    d.tags = vec![CardTag::Attack];
    d
}

fn black_oath_def() -> CardDef {
    let mut d = warlock_sc(BLACK_OATH, "Black Oath", "hexblade", 1, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Empowered, stacks: 2 },
        Effect::GainBlock { amount: 8 },
    ], "Gain 2 Empowered. Gain 8 block. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

// ── Void: Void Debuff synergy ───────────────────────────────────────

fn void_gaze_def() -> CardDef {
    let mut d = warlock_sc(VOID_GAZE, "Void Gaze", "void", 1, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
    ], "Apply 1 Weakened to ALL enemies. Concentration.");
    d.concentration = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn starless_whisper_def() -> CardDef {
    let mut d = warlock_sc(STARLESS_WHISPER, "Starless Whisper", "void", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Marked, stacks: 1 },
    ], "Draw 2 cards. Discard 1. Apply 1 Marked to ALL enemies.");
    // Note: DrawAndDiscard would be more accurate but DrawCards + discard 1 not directly possible
    // Use DrawAndDiscard effect instead
    d.effects = vec![
        Effect::DrawAndDiscard { draw: 2, discard: 1 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Marked, stacks: 1 },
    ];
    d.tags = vec![CardTag::Skill];
    d
}

fn entropy_field_def() -> CardDef {
    let mut d = warlock_sc(ENTROPY_FIELD, "Entropy Field", "void", 1, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
    ], "Apply 1 Weakened and 1 Threatened to ALL enemies.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Void: Void Annihilation synergy ─────────────────────────────────

fn oblivion_tide_def() -> CardDef {
    let mut d = warlock_sc(OBLIVION_TIDE, "Oblivion Tide", "void", 2, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
        Effect::DealDamagePerDebuffOnTarget { target: Target::AllEnemies, base: 8, per_debuff: 8 },
    ], "Apply 1 Weakened to ALL. Deal 8 + 8 per debuff type to ALL enemies. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack, CardTag::Skill];
    d
}

fn cosmic_extinction_def() -> CardDef {
    let mut d = warlock_sc(COSMIC_EXTINCTION, "Cosmic Extinction", "void", 0, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Hexed, stacks: 1 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 2 },
    ], "Apply Hexed, 2 Weakened, and 2 Threatened to ALL enemies. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ══════════════════════════════════════════════════════════════════════
// WIZARD CARDS
// ══════════════════════════════════════════════════════════════════════

// ── Wizard shared cards ────────────────────────────────────────────────

fn fire_bolt_def() -> CardDef {
    let mut d = wizard(FIRE_BOLT, "Fire Bolt", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Bleeding, stacks: 2 },
    ], "Deal 10 damage. Apply 2 Bleeding.");
    d.tags = vec![CardTag::Attack];
    d
}

fn mage_armor_def() -> CardDef {
    let mut d = wizard(MAGE_ARMOR, "Mage Armor", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 10 block. Gain 1 Arcane Charge.");
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

fn arcane_intellect_def() -> CardDef {
    let mut d = wizard(ARCANE_INTELLECT, "Arcane Intellect", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Draw 2. Gain 1 Arcane Charge. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn shield_spell_def() -> CardDef {
    let mut d = wizard(SHIELD_SPELL, "Shield", 1, vec![
        Effect::GainBlock { amount: 12 },
        Effect::ApplyStatusIfPlayerHasStatus {
            status: StatusType::Fortified,
            stacks: 2,
            require_status: StatusType::ArcaneCharge,
            require_stacks: 2,
        },
    ], "Gain 12 block. If Arcane Charge >= 2: gain 2 Fortified.");
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

// ── Wizard themed basics (subclass starters) ──────────────────────────

fn ember_bolt_def() -> CardDef {
    let mut d = wizard_sc(EMBER_BOLT, "Ember Bolt", "evocation", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 7 },
    ], "Deal 7 damage.");
    d.tags = vec![CardTag::Attack];
    d
}

fn arcane_ward_def() -> CardDef {
    let mut d = wizard_sc(ARCANE_WARD, "Arcane Ward", "abjuration", 1, vec![
        Effect::GainBlock { amount: 8 },
    ], "Gain 8 block.");
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

fn prescience_def() -> CardDef {
    let mut d = wizard_sc(PRESCIENCE, "Prescience", "divination", 0, vec![
        Effect::DrawCards { count: 1 },
    ], "Draw 1. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Evocation subclass progression ────────────────────────────────────

fn fireball_def() -> CardDef {
    let mut d = wizard_sc(FIREBALL, "Fireball", "evocation", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 12 },
    ], "Deal 12 damage to ALL enemies. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn scorching_ray_def() -> CardDef {
    let mut d = wizard_sc(SCORCHING_RAY, "Scorching Ray", "evocation", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 4 },
    ], "Deal 4 damage 3 times.");
    d.tags = vec![CardTag::Attack];
    d
}

fn lightning_bolt_def() -> CardDef {
    let mut d = wizard_sc(LIGHTNING_BOLT, "Lightning Bolt", "evocation", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 15 },
    ], "Deal 15 damage. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn flame_shield_def() -> CardDef {
    let mut d = wizard_sc(FLAME_SHIELD, "Flame Shield", "evocation", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 2 },
    ], "Gain 8 block. Gain 2 Barbed.");
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

// ── Abjuration subclass progression ───────────────────────────────────

fn counterspell_def() -> CardDef {
    let mut d = wizard_sc(COUNTERSPELL, "Counterspell", "abjuration", 1, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::SanctifiedShield, stacks: 1 },
    ], "Negate the next enemy attack. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn ward_def() -> CardDef {
    let mut d = wizard_sc(WARD, "Ward", "abjuration", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Fortified, stacks: 2 },
    ], "Gain 10 block. Gain 2 Fortified.");
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

fn dispel_def() -> CardDef {
    let mut d = wizard_sc(DISPEL, "Dispel", "abjuration", 0, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Empowered, stacks: -2 },
    ], "Remove 2 Empowered from all enemies. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn globe_of_protection_def() -> CardDef {
    let mut d = wizard_sc(GLOBE_OF_PROTECTION, "Globe of Protection", "abjuration", 2, vec![
        Effect::GainBlock { amount: 20 },
    ], "Gain 20 block. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill, CardTag::Defense];
    d
}

// ── Divination subclass progression ───────────────────────────────────

fn foresight_def() -> CardDef {
    let mut d = wizard_sc(FORESIGHT, "Foresight", "divination", 0, vec![
        Effect::DrawAndDiscard { draw: 3, discard: 1 },
    ], "Draw 3, discard 1. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn scrying_def() -> CardDef {
    let mut d = wizard_sc(SCRYING, "Scrying", "divination", 1, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Marked, stacks: 1 },
    ], "Draw 2. Apply 1 Marked to all enemies.");
    d.tags = vec![CardTag::Skill];
    d
}

fn portent_def() -> CardDef {
    let mut d = wizard_sc(PORTENT, "Portent", "divination", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
    ], "Gain 2 Arcane Charge. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn time_stop_def() -> CardDef {
    let mut d = wizard_sc(TIME_STOP, "Time Stop", "divination", 2, vec![
        Effect::GainEnergy { amount: 3 },
        Effect::DrawCards { count: 3 },
    ], "Gain 3 energy. Draw 3. Exhaust.");
    d.exhaust = true;
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Wizard capstone ───────────────────────────────────────────────────

fn wish_def() -> CardDef {
    let mut d = wizard(WISH, "Wish", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 3 },
        Effect::DrawCards { count: 3 },
        Effect::Heal { amount: 10 },
    ], "Gain 3 Arcane Charge. Draw 3. Heal 10. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ══════════════════════════════════════════════════════════════════════
// PALADIN
// ══════════════════════════════════════════════════════════════════════

// ── Shared Paladin cards ────────────────────────────────────────────

fn holy_strike_def() -> CardDef {
    let mut d = paladin(HOLY_STRIKE, "Holy Strike", 1, vec![
        Effect::DealDamageSpendSmite { target: Target::SingleEnemy, base: 8, smite_cost: 1, bonus: 16 },
        Effect::GainBlock { amount: 4 },
    ], "Deal 8 damage + 4 block. Spend 1 Smite: deal 16 instead.");
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn lay_on_hands_def() -> CardDef {
    let mut d = paladin(LAY_ON_HANDS, "Lay on Hands", 1, vec![
        Effect::Heal { amount: 12 },
        Effect::GainBlock { amount: 4 },
    ], "Heal 12 HP. Gain 4 block. Exhaust.");
    d.card_type = CardType::Consumable;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn prayer_of_valor_def() -> CardDef {
    let mut d = paladin(PRAYER_OF_VALOR, "Prayer of Valor", 0, vec![
        Effect::GainEnergy { amount: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Smite, stacks: 1 },
    ], "Gain 1 energy + 1 Smite. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn consecration_def() -> CardDef {
    let mut d = paladin(CONSECRATION, "Consecration", 1, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
        Effect::GainBlock { amount: 6 },
    ], "Apply 1 Threatened to ALL enemies. Gain 6 block.");
    d.tags = vec![CardTag::Defense];
    d
}

fn divine_bulwark_def() -> CardDef {
    let mut d = paladin(DIVINE_BULWARK, "Divine Bulwark", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::BlockRetention, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 2 },
    ], "Gain Sanctified Shield (Block Retention + 2 Armored). Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

// ── Devotion style ──────────────────────────────────────────────────

fn warding_slash_def() -> CardDef {
    let mut d = paladin_sc(WARDING_SLASH, "Warding Slash", "devotion", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::GainBlockSpendSmite { base: 5, smite_cost: 1, bonus: 13 },
    ], "Deal 10 damage + 5 block. Spend 1 Smite: 13 block instead.");
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn shield_of_faith_def() -> CardDef {
    let mut d = paladin_sc(SHIELD_OF_FAITH, "Shield of Faith", "devotion", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 1 },
    ], "Gain 10 block + 1 Armored.");
    d.tags = vec![CardTag::Defense];
    d
}

fn bastion_drive_def() -> CardDef {
    let mut d = paladin_sc(BASTION_DRIVE, "Bastion Drive", "devotion", 2, vec![
        Effect::DealDamageSpendSmite { target: Target::SingleEnemy, base: 14, smite_cost: 1, bonus: 22 },
        Effect::GainBlock { amount: 12 },
    ], "Deal 14 damage + 12 block. Spend 1 Smite: deal 22 instead.");
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn holy_bastion_def() -> CardDef {
    let mut d = paladin_sc(HOLY_BASTION, "Holy Bastion", "devotion", 1, vec![
        Effect::GainBlock { amount: 12 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 2 },
    ], "Gain 12 block + 2 Armored.");
    d.tags = vec![CardTag::Defense];
    d
}

// ── Vengeance style ─────────────────────────────────────────────────

fn avenging_strike_def() -> CardDef {
    let mut d = paladin_sc(AVENGING_STRIKE, "Avenging Strike", "vengeance", 1, vec![
        Effect::DealDamageSpendSmite { target: Target::SingleEnemy, base: 12, smite_cost: 1, bonus: 20 },
    ], "Deal 12 damage. Spend 1 Smite: deal 20 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

fn blade_of_wrath_def() -> CardDef {
    let mut d = paladin_sc(BLADE_OF_WRATH, "Blade of Wrath", "vengeance", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Empowered, stacks: 2 },
    ], "Deal 10 damage. Gain 2 Empowered.");
    d.tags = vec![CardTag::Attack];
    d
}

fn divine_judgment_def() -> CardDef {
    let mut d = paladin_sc(DIVINE_JUDGMENT, "Divine Judgment", "vengeance", 2, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 20 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Smite, stacks: 2 },
    ], "Deal 20 damage. Gain 2 Smite. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn zealous_rush_def() -> CardDef {
    let mut d = paladin_sc(ZEALOUS_RUSH, "Zealous Rush", "vengeance", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Empowered, stacks: 1 },
    ], "Draw 2 cards. Gain 1 Empowered. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Radiance style ──────────────────────────────────────────────────

fn radiant_burst_def() -> CardDef {
    let mut d = paladin_sc(RADIANT_BURST, "Radiant Burst", "radiance", 1, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 8 },
        Effect::GainBlock { amount: 4 },
    ], "Deal 8 damage to ALL enemies. Gain 4 block.");
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

fn beacon_of_light_def() -> CardDef {
    let mut d = paladin_sc(BEACON_OF_LIGHT, "Beacon of Light", "radiance", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::Heal { amount: 6 },
    ], "Gain 8 block. Heal 6 HP.");
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn solar_flare_def() -> CardDef {
    let mut d = paladin_sc(SOLAR_FLARE, "Solar Flare", "radiance", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 15 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 2 },
    ], "Deal 15 damage to ALL enemies. Apply 2 Threatened to ALL. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn dawns_blessing_def() -> CardDef {
    let mut d = paladin_sc(DAWNS_BLESSING, "Dawn's Blessing", "radiance", 0, vec![
        Effect::Heal { amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Heal 8 HP. Draw 1 card. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Paladin Capstone ────────────────────────────────────────────────

fn avatar_of_radiance_def() -> CardDef {
    let mut d = paladin(AVATAR_OF_RADIANCE, "Avatar of Radiance", 0, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 12 },
        Effect::GainBlock { amount: 15 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Smite, stacks: 2 },
    ], "Deal 12 damage to ALL. Gain 15 block. Gain 2 Smite. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.innate = true;
    d.tags = vec![CardTag::Attack, CardTag::Defense];
    d
}

// ══════════════════════════════════════════════════════════════════════
// ROGUE CLASS CARDS
// ══════════════════════════════════════════════════════════════════════

// ── Rogue helper constructors ─────────────────────────────────────────

fn rogue(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Rogue),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn rogue_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Rogue),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

// ── Rogue class card ─────────────────────────────────────────────────

fn tricks_of_the_trade_def() -> CardDef {
    let mut d = rogue(TRICKS_OF_THE_TRADE, "Tricks of the Trade", 1, vec![
        Effect::Heal { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 5 },
    ], "Heal 8 HP. Gain 5 Artifice.");
    d.card_type = CardType::Consumable;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Rogue shared auto-grants ─────────────────────────────────────────

fn sneak_attack_def() -> CardDef {
    let mut d = rogue(SNEAK_ATTACK, "Sneak Attack", 1, vec![
        Effect::DealDamageSpendArtifice { target: Target::SingleEnemy, base: 10, artifice_cost: 10, bonus: 15 },
    ], "Deal 10 damage. Spend 10 Artifice: deal 15 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

fn cunning_action_def() -> CardDef {
    let mut d = rogue(CUNNING_ACTION, "Cunning Action", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 3 },
    ], "Draw 2 cards. Gain 3 Artifice. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn rogue_evasion_def() -> CardDef {
    let mut d = rogue(ROGUE_EVASION, "Evasion", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 3 },
    ], "Gain 10 block. Gain 3 Artifice.");
    d.tags = vec![CardTag::Defense];
    d
}

fn preparation_def() -> CardDef {
    let mut d = rogue(PREPARATION, "Preparation", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 5 },
        Effect::DrawCards { count: 1 },
    ], "Gain 5 Artifice. Draw 1 card. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Assassin synergy: assassination ──────────────────────────────────

fn assassinate_def() -> CardDef {
    let mut d = rogue_sc(ASSASSINATE, "Assassinate", "assassin", 1, vec![
        Effect::DealDamageSpendArtifice { target: Target::SingleEnemy, base: 12, artifice_cost: 10, bonus: 18 },
    ], "Deal 12 damage. Spend 10 Artifice: deal 18 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

fn rogue_shadow_step_def() -> CardDef {
    let mut d = rogue_sc(ROGUE_SHADOW_STEP, "Shadow Step", "assassin", 0, vec![
        Effect::DrawCards { count: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Empowered, stacks: 1 },
    ], "Draw 1 card. Gain 2 Artifice. Gain 1 Empowered. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn death_mark_def() -> CardDef {
    let mut d = rogue_sc(DEATH_MARK, "Death Mark", "assassin", 1, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Marked, stacks: 2 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Threatened, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 5 },
    ], "Apply 2 Marked and 2 Threatened. Gain 5 Artifice.");
    d.tags = vec![CardTag::Skill];
    d
}

fn killing_blow_def() -> CardDef {
    let mut d = rogue_sc(KILLING_BLOW, "Killing Blow", "assassin", 2, vec![
        Effect::DealDamageSpendArtifice { target: Target::SingleEnemy, base: 20, artifice_cost: 10, bonus: 30 },
    ], "Deal 20 damage. Spend 10 Artifice: deal 30 instead. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d.rarity = Rarity::Uncommon;
    d
}

// ── Pirate synergy: pirate_blade / pirate_trick ─────────────────────

fn cutlass_flurry_def() -> CardDef {
    let mut d = rogue_sc(CUTLASS_FLURRY, "Cutlass Flurry", "pirate", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 5 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 4 },
    ], "Deal 5 damage twice. Gain 4 Artifice.");
    d.tags = vec![CardTag::Attack];
    d
}

fn grappling_hook_def() -> CardDef {
    let mut d = rogue_sc(GRAPPLING_HOOK, "Grappling Hook", "pirate", 0, vec![
        Effect::DrawAndDiscard { draw: 2, discard: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 3 },
    ], "Draw 2, discard 1. Gain 3 Artifice.");
    d.tags = vec![CardTag::Skill];
    d
}

fn dirty_fighting_def() -> CardDef {
    let mut d = rogue_sc(DIRTY_FIGHTING, "Dirty Fighting", "pirate", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 3 },
    ], "Deal 8 damage. Apply 1 Weakened. Gain 3 Artifice.");
    d.tags = vec![CardTag::Attack];
    d
}

fn broadside_def() -> CardDef {
    let mut d = rogue_sc(BROADSIDE, "Broadside", "pirate", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 12 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 4 },
    ], "Deal 12 damage to ALL enemies. Gain 4 Artifice. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d.rarity = Rarity::Uncommon;
    d
}

// ── Trickster synergy: trickster_smoke / trickster_finesse ──────────

fn smoke_bomb_def() -> CardDef {
    let mut d = rogue_sc(SMOKE_BOMB, "Smoke Bomb", "trickster", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 5 },
    ], "Gain 8 block. Apply 1 Weakened to ALL enemies. Gain 5 Artifice.");
    d.tags = vec![CardTag::Defense];
    d
}

fn misdirection_def() -> CardDef {
    let mut d = rogue_sc(MISDIRECTION, "Misdirection", "trickster", 0, vec![
        Effect::GainBlock { amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Gain 8 block. Draw 1 card. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Defense];
    d
}

fn fan_of_knives_def() -> CardDef {
    let mut d = rogue_sc(FAN_OF_KNIVES, "Fan of Knives", "trickster", 1, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 6 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Artifice, stacks: 4 },
    ], "Deal 6 damage to ALL enemies. Gain 4 Artifice.");
    d.tags = vec![CardTag::Attack];
    d
}

fn ace_in_the_hole_def() -> CardDef {
    let mut d = rogue_sc(ACE_IN_THE_HOLE, "Ace in the Hole", "trickster", 1, vec![
        Effect::DealDamageSpendArtifice { target: Target::SingleEnemy, base: 15, artifice_cost: 10, bonus: 22 },
        Effect::DrawCards { count: 2 },
    ], "Deal 15 damage. Draw 2 cards. Spend 10 Artifice: deal 22 instead. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d.rarity = Rarity::Uncommon;
    d
}

/// Map background ID to its card ID.
pub fn background_card_id(background: &str) -> &'static str {
    match background {
        "soldier" => BATTLE_DISCIPLINE,
        "scholar" => STUDIED_ANALYSIS,
        "acolyte" => MINOR_BLESSING,
        "urchin" => DIRTY_TRICK,
        "noble" => COMMANDING_PRESENCE,
        "outlander" => SURVIVAL_INSTINCT,
        "artisan" => IMPROVISED_WEAPON,
        "sailor" => SEA_LEGS,
        "criminal" => BACKSTAB,
        "hermit" => INNER_FOCUS,
        "merchant" => RESOURCEFUL,
        "entertainer" => DAZZLE,
        _ => panic!("unknown background: {}", background),
    }
}

// ── Druid helpers ──────────────────────────────────────────────────────

fn druid(id: &str, name: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Druid),
        subclass: None,
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

fn druid_sc(id: &str, name: &str, subclass: &str, cost: i32, effects: Vec<Effect>, desc: &str) -> CardDef {
    CardDef {
        id: id.into(),
        name: name.into(),
        rarity: Rarity::Common,
        cost,
        card_type: CardType::Spell,
        exhaust: false,
        effects,
        description: desc.into(),
        class: Some(Class::Druid),
        subclass: Some(subclass.into()),
        tags: vec![],
        recycle: false,
        concentration: false,
        innate: false,
        milestone: false,
        inherent: false,
    }
}

// ── Druid form-entry cantrips ──────────────────────────────────────────

fn wild_shape_bear_def() -> CardDef {
    let mut d = druid_sc(WILD_SHAPE_BEAR, "Wild Shape: Bear", "bear", 0, vec![
        Effect::EnterWildShape { form: StatusType::WildShapeBear },
    ], "Enter Bear form. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn wild_shape_eagle_def() -> CardDef {
    let mut d = druid_sc(WILD_SHAPE_EAGLE, "Wild Shape: Eagle", "eagle", 0, vec![
        Effect::EnterWildShape { form: StatusType::WildShapeEagle },
    ], "Enter Eagle form. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn wild_shape_wolf_def() -> CardDef {
    let mut d = druid_sc(WILD_SHAPE_WOLF, "Wild Shape: Wolf", "wolf", 0, vec![
        Effect::EnterWildShape { form: StatusType::WildShapeWolf },
    ], "Enter Wolf form. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Druid shared cards ────────────────────────────────────────────────

fn natures_wrath_def() -> CardDef {
    let mut d = druid(NATURES_WRATH, "Nature's Wrath", 1, vec![
        Effect::DealDamageIfInWildShape { target: Target::SingleEnemy, base: 10, bonus: 5 },
    ], "Deal 10 damage. In Wild Shape: deal 15.");
    d.tags = vec![CardTag::Attack];
    d
}

fn bark_skin_def() -> CardDef {
    let mut d = druid(BARK_SKIN, "Bark Skin", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 3 },
    ], "Gain 10 block. Gain 3 Armored.");
    d.tags = vec![CardTag::Defense];
    d
}

fn rejuvenation_def() -> CardDef {
    let mut d = druid(REJUVENATION, "Rejuvenation", 1, vec![
        Effect::Heal { amount: 8 },
        Effect::DrawCards { count: 1 },
    ], "Heal 8 HP. Draw 1 card. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Bear synergy cards ────────────────────────────────────────────────

fn bear_maul_def() -> CardDef {
    let mut d = druid_sc(BEAR_MAUL, "Bear Maul", "bear", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 12 },
        Effect::GainBlockWithFormBonus { base: 0, status: StatusType::WildShapeBear, bonus: 6 },
    ], "Deal 12 damage. In Bear form: gain 6 block.");
    d.tags = vec![CardTag::Attack];
    d
}

fn thick_hide_def() -> CardDef {
    let mut d = druid_sc(THICK_HIDE, "Thick Hide", "bear", 0, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 2 },
    ], "Gain 8 block. Gain 2 Armored. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Defense];
    d
}

fn ursine_charge_def() -> CardDef {
    let mut d = druid_sc(URSINE_CHARGE, "Ursine Charge", "bear", 2, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 18 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Threatened, stacks: 1 },
    ], "Deal 18 damage. Apply 1 Threatened. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn primal_roar_def() -> CardDef {
    let mut d = druid_sc(PRIMAL_ROAR, "Primal Roar", "bear", 1, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
        Effect::GainBlock { amount: 8 },
    ], "Apply 1 Threatened to ALL enemies. Gain 8 block.");
    d.tags = vec![CardTag::Defense];
    d
}

// ── Eagle synergy cards ───────────────────────────────────────────────

fn eagle_dive_def() -> CardDef {
    let mut d = druid_sc(EAGLE_DIVE, "Eagle Dive", "eagle", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::DrawCardsWithFormBonus { base_draw: 1, status: StatusType::WildShapeEagle, bonus_draw: 1 },
    ], "Deal 10 damage. Draw 1. In Eagle form: draw 2.");
    d.tags = vec![CardTag::Attack];
    d
}

fn swooping_strike_def() -> CardDef {
    let mut d = druid_sc(SWOOPING_STRIKE, "Swooping Strike", "eagle", 0, vec![
        Effect::DealDamageWithFormBonus { target: Target::SingleEnemy, base: 6, status: StatusType::WildShapeEagle, bonus: 3 },
    ], "Deal 6 damage. In Eagle form: deal 9.");
    d.tags = vec![CardTag::Attack];
    d
}

fn wind_rider_def() -> CardDef {
    let mut d = druid_sc(WIND_RIDER, "Wind Rider", "eagle", 1, vec![
        Effect::DrawAndDiscard { draw: 3, discard: 1 },
        Effect::GainBlock { amount: 5 },
    ], "Draw 3. Discard 1. Gain 5 block.");
    d.tags = vec![CardTag::Skill];
    d
}

fn tempest_talons_def() -> CardDef {
    let mut d = druid_sc(TEMPEST_TALONS, "Tempest Talons", "eagle", 2, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::DealDamageIfPlayerHasStatus { target: Target::SingleEnemy, status: StatusType::WildShapeEagle, amount: 8 },
    ], "Deal 8 damage x2. In Eagle form: 8 x3. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Wolf synergy cards ────────────────────────────────────────────────

fn pack_tactics_def() -> CardDef {
    let mut d = druid_sc(PACK_TACTICS, "Pack Tactics", "wolf", 1, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 8 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
    ], "Deal 8 damage to ALL. Apply 1 Weakened to ALL.");
    d.tags = vec![CardTag::Attack];
    d
}

fn howl_def() -> CardDef {
    let mut d = druid_sc(HOWL, "Howl", "wolf", 0, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
        Effect::DrawCards { count: 1 },
    ], "Apply 1 Threatened to ALL. Draw 1. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn coordinated_strike_def() -> CardDef {
    let mut d = druid_sc(COORDINATED_STRIKE, "Coordinated Strike", "wolf", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 10, bonus: 5 },
    ], "Deal 10 damage. If enemy has debuff: deal 15.");
    d.tags = vec![CardTag::Attack];
    d
}

fn alphas_command_def() -> CardDef {
    let mut d = druid_sc(ALPHAS_COMMAND, "Alpha's Command", "wolf", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 12 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
    ], "Deal 12 damage to ALL. Weakened ALL. Threatened ALL. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Druid concentration spells ────────────────────────────────────────

fn moonbeam_def() -> CardDef {
    let mut d = druid(MOONBEAM, "Moonbeam", 1, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 8 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 1 },
    ], "Deal 8 damage to ALL. Apply 1 Weakened ALL. Concentration.");
    d.concentration = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn entangle_def() -> CardDef {
    let mut d = druid(ENTANGLE, "Entangle", 1, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
    ], "Apply 2 Weakened + 1 Threatened to ALL. Concentration.");
    d.concentration = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Druid capstone ────────────────────────────────────────────────────

fn archdruid_def() -> CardDef {
    let mut d = druid(ARCHDRUID, "Archdruid", 0, vec![
        Effect::EnterWildShape { form: StatusType::WildShapeBear },
        Effect::GainBlock { amount: 15 },
        Effect::Heal { amount: 8 },
    ], "Enter Bear form. Gain 15 block. Heal 8 HP. Innate. Exhaust.");
    d.innate = true;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ══════════════════════════════════════════════════════════════════════
// SORCERER
// ══════════════════════════════════════════════════════════════════════

// ── Wild Mage starter cards ────────────────────────────────────────────

fn spark_bolt_def() -> CardDef {
    let mut d = sorcerer_sc(SPARK_BOLT, "Spark Bolt", "wild_mage", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 8 damage. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack];
    d
}

fn channel_def() -> CardDef {
    let mut d = sorcerer_sc(CHANNEL, "Channel", "wild_mage", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
    ], "Gain 2 Sorcery Points. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn surge_wave_def() -> CardDef {
    let mut d = sorcerer_sc(SURGE_WAVE, "Surge Wave", "wild_mage", 1, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 6 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 6 damage to ALL enemies. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack];
    d
}

fn arcane_shield_def() -> CardDef {
    let mut d = sorcerer_sc(ARCANE_SHIELD, "Arcane Shield", "wild_mage", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 10 block. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn wild_surge_def() -> CardDef {
    let mut d = sorcerer_sc(WILD_SURGE, "Wild Surge", "wild_mage", 0, vec![
        Effect::DrawCards { count: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Draw 1. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Skill];
    d
}

// ── Draconic Bloodline starter cards ──────────────────────────────────

fn claw_strike_def() -> CardDef {
    let mut d = sorcerer_sc(CLAW_STRIKE, "Claw Strike", "draconic", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 10 damage. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack];
    d
}

fn scale_ward_def() -> CardDef {
    let mut d = sorcerer_sc(SCALE_WARD, "Scale Ward", "draconic", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 1 },
    ], "Gain 10 block. Gain 1 Barbed.");
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn draconic_focus_def() -> CardDef {
    let mut d = sorcerer_sc(DRACONIC_FOCUS, "Draconic Focus", "draconic", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
    ], "Gain 2 Sorcery Points. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn dragons_resilience_def() -> CardDef {
    let mut d = sorcerer_sc(DRAGONS_RESILIENCE, "Dragon's Resilience", "draconic", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 1 },
    ], "Gain 8 block. Gain 1 Armored.");
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn draconic_bolt_def() -> CardDef {
    let mut d = sorcerer_sc(DRACONIC_BOLT, "Draconic Bolt", "draconic", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 8 damage. Gain 1 Sorcery Point. Recycle.");
    d.recycle = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Shadow Sorcerer starter cards ─────────────────────────────────────

fn shadow_bolt_def() -> CardDef {
    let mut d = sorcerer_sc(SHADOW_BOLT, "Shadow Bolt", "shadow_sorcerer", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 1 },
    ], "Deal 8 damage. Apply 1 Weakened.");
    d.tags = vec![CardTag::Attack];
    d
}

fn dark_shroud_def() -> CardDef {
    let mut d = sorcerer_sc(DARK_SHROUD, "Dark Shroud", "shadow_sorcerer", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 10 block. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn siphon_def() -> CardDef {
    let mut d = sorcerer_sc(SIPHON, "Siphon", "shadow_sorcerer", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 2 },
    ], "Gain 2 Sorcery Points. Gain 2 Mending. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn night_veil_def() -> CardDef {
    let mut d = sorcerer_sc(NIGHT_VEIL, "Night Veil", "shadow_sorcerer", 0, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Frightened, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Apply 1 Frightened to an enemy. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Skill];
    d
}

fn umbral_strike_def() -> CardDef {
    let mut d = sorcerer_sc(UMBRAL_STRIKE, "Umbral Strike", "shadow_sorcerer", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 7, bonus: 7 },
    ], "Deal 7 damage. If target is debuffed: deal 14 instead.");
    d.tags = vec![CardTag::Attack];
    d
}

// ── Sorcerer shared pool cards ─────────────────────────────────────────

fn arcane_pulse_def() -> CardDef {
    let mut d = sorcerer(ARCANE_PULSE, "Arcane Pulse", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
    ], "Deal 8 damage. Gain 2 Sorcery Points.");
    d.tags = vec![CardTag::Attack];
    d
}

fn mana_infusion_def() -> CardDef {
    let mut d = sorcerer(MANA_INFUSION, "Mana Infusion", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 3 },
    ], "Gain 3 Sorcery Points. Exhaust.");
    d.rarity = Rarity::Uncommon;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Power];
    d
}

fn spell_surge_def() -> CardDef {
    let mut d = sorcerer(SPELL_SURGE, "Spell Surge", 1, vec![
        Effect::GainBlock { amount: 6 },
        Effect::DrawCards { count: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 6 block. Draw 1. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Defense, CardTag::Skill];
    d
}

fn overload_def() -> CardDef {
    let mut d = sorcerer(OVERLOAD, "Overload", 0, vec![
        Effect::DealDamagePerPlayerStatus { target: Target::SingleEnemy, status: StatusType::ArcaneCharge, per_stack: 3 },
    ], "Deal 3 damage per Sorcery Point. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Wild Mage pool: Surge synergy ─────────────────────────────────────

fn volatile_burst_def() -> CardDef {
    let mut d = sorcerer_sc(VOLATILE_BURST, "Volatile Burst", "wild_mage", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 1 },
    ], "Deal 10 damage to ALL enemies. Gain 2 Sorcery Points. Apply 1 Threatened to ALL.");
    d.tags = vec![CardTag::Attack];
    d
}

fn chain_lightning_def() -> CardDef {
    let mut d = sorcerer_sc(CHAIN_LIGHTNING, "Chain Lightning", "wild_mage", 2, vec![
        Effect::DealDamage { target: Target::AllEnemies, amount: 14 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Bleeding, stacks: 2 },
    ], "Deal 14 damage to ALL enemies. Apply 2 Bleeding to ALL.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn surge_nova_def() -> CardDef {
    let mut d = sorcerer_sc(SURGE_NOVA, "Surge Nova", "wild_mage", 1, vec![
        Effect::DealDamagePerPlayerStatus { target: Target::AllEnemies, status: StatusType::ArcaneCharge, per_stack: 2 },
    ], "Deal 2 damage per Sorcery Point to ALL enemies. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn wild_discharge_def() -> CardDef {
    let mut d = sorcerer_sc(WILD_DISCHARGE, "Wild Discharge", "wild_mage", 0, vec![
        Effect::DrawCards { count: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
        Effect::LoseHp { amount: 2 },
    ], "Draw 2. Gain 1 Sorcery Point. Lose 2 HP.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn sorcerer_cataclysm_def() -> CardDef {
    let mut d = sorcerer_sc(SORCERER_CATACLYSM, "Cataclysm", "wild_mage", 2, vec![
        Effect::DealDamagePerPlayerStatus { target: Target::AllEnemies, status: StatusType::ArcaneCharge, per_stack: 3 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 2 },
    ], "Deal 3 damage per Sorcery Point to ALL enemies. Apply 2 Weakened to ALL. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Wild Mage pool: Chaos synergy ─────────────────────────────────────

fn chaos_bolt_def() -> CardDef {
    let mut d = sorcerer_sc(CHAOS_BOLT, "Chaos Bolt", "wild_mage", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 8, bonus: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 8 damage. If target is debuffed: deal 16 instead. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack];
    d
}

fn unraveling_hex_def() -> CardDef {
    let mut d = sorcerer_sc(UNRAVELING_HEX, "Unraveling Hex", "wild_mage", 1, vec![
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Weakened, stacks: 2 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Threatened, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Apply 2 Weakened + 1 Threatened to enemy. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn entropy_cascade_def() -> CardDef {
    let mut d = sorcerer_sc(ENTROPY_CASCADE, "Entropy Cascade", "wild_mage", 1, vec![
        Effect::DealDamagePerDebuffOnTarget { target: Target::SingleEnemy, base: 6, per_debuff: 6 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 6 + 6 per debuff type on target. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Attack];
    d
}

fn sorcerer_pandemonium_def() -> CardDef {
    let mut d = sorcerer_sc(SORCERER_PANDEMONIUM, "Pandemonium", "wild_mage", 1, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 2 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Bleeding, stacks: 3 },
    ], "Apply 2 Weakened + 2 Threatened + 3 Bleeding to ALL enemies. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

// ── Draconic pool: Heritage synergy ───────────────────────────────────

fn dragon_scale_def() -> CardDef {
    let mut d = sorcerer_sc(DRAGON_SCALE, "Dragon Scale", "draconic", 1, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 10 block. Gain 1 Barbed. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Defense];
    d
}

fn ancestral_resilience_def() -> CardDef {
    let mut d = sorcerer_sc(ANCESTRAL_RESILIENCE, "Ancestral Resilience", "draconic", 1, vec![
        Effect::GainBlock { amount: 8 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 8 block. Gain 2 Armored. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

fn scaled_rebuke_def() -> CardDef {
    let mut d = sorcerer_sc(SCALED_REBUKE, "Scaled Rebuke", "draconic", 1, vec![
        Effect::GainBlockPerStatusStack { status: StatusType::Barbed, multiplier: 2 },
        Effect::DealDamageIfDamagedLastTurn { target: Target::SingleEnemy, amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Gain 2 block per Barbed stack. If damaged last turn: deal 10 damage. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Rare;
    d.tags = vec![CardTag::Defense, CardTag::Attack];
    d
}

fn primordial_resilience_def() -> CardDef {
    let mut d = sorcerer_sc(PRIMORDIAL_RESILIENCE, "Primordial Resilience", "draconic", 0, vec![
        Effect::GainBlock { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Barbed, stacks: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Armored, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
    ], "Gain 10 block. Gain 3 Barbed + 2 Armored + 2 Sorcery Points. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Defense, CardTag::Power];
    d
}

// ── Draconic pool: Wrath synergy ──────────────────────────────────────

fn sorcerer_dragons_breath_def() -> CardDef {
    let mut d = sorcerer_sc(SORCERER_DRAGONS_BREATH, "Dragon's Breath", "draconic", 2, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 18 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Bleeding, stacks: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 18 damage. Apply 3 Bleeding. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn draconic_surge_def() -> CardDef {
    let mut d = sorcerer_sc(DRACONIC_SURGE, "Draconic Surge", "draconic", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 12 },
        Effect::ApplyStatusIfPlayerHasStatus {
            status: StatusType::ArcaneCharge,
            stacks: 2,
            require_status: StatusType::ArcaneCharge,
            require_stacks: 4,
        },
    ], "Deal 12 damage. If Sorcery Points >= 4: gain 2 more.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn ancient_power_def() -> CardDef {
    let mut d = sorcerer_sc(ANCIENT_POWER, "Ancient Power", "draconic", 2, vec![
        Effect::DealDamagePerPlayerStatus { target: Target::SingleEnemy, status: StatusType::ArcaneCharge, per_stack: 4 },
    ], "Deal 4 damage per Sorcery Point. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn dragonfire_ascension_def() -> CardDef {
    let mut d = sorcerer_sc(DRAGONFIRE_ASCENSION, "Dragonfire Ascension", "draconic", 2, vec![
        Effect::DealDamagePerPlayerStatus { target: Target::SingleEnemy, status: StatusType::ArcaneCharge, per_stack: 5 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Threatened, stacks: 2 },
    ], "Deal 5 damage per Sorcery Point. Apply 2 Threatened to ALL enemies. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

// ── Shadow Sorcerer pool: Drain synergy ───────────────────────────────

fn shadow_tap_def() -> CardDef {
    let mut d = sorcerer_sc(SHADOW_TAP, "Shadow Tap", "shadow_sorcerer", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 2 },
    ], "Gain 2 Sorcery Points. Gain 2 Mending. Exhaust.");
    d.exhaust = true;
    d.tags = vec![CardTag::Skill];
    d
}

fn necrotic_siphon_def() -> CardDef {
    let mut d = sorcerer_sc(NECROTIC_SIPHON, "Necrotic Siphon", "shadow_sorcerer", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::Heal { amount: 4 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 8 damage. Heal 4 HP. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack];
    d
}

fn life_drain_def() -> CardDef {
    let mut d = sorcerer_sc(LIFE_DRAIN, "Life Drain", "shadow_sorcerer", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 8, bonus: 6 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 3 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 8 damage (14 if debuffed). Gain 3 Mending. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Attack];
    d
}

fn void_feast_def() -> CardDef {
    let mut d = sorcerer_sc(VOID_FEAST, "Void Feast", "shadow_sorcerer", 2, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 14 },
        Effect::Heal { amount: 10 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
    ], "Deal 14 damage. Heal 10 HP. Gain 2 Sorcery Points. Exhaust.");
    d.rarity = Rarity::Rare;
    d.exhaust = true;
    d.tags = vec![CardTag::Attack];
    d
}

fn eternal_hunger_def() -> CardDef {
    let mut d = sorcerer_sc(ETERNAL_HUNGER, "Eternal Hunger", "shadow_sorcerer", 1, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::Mending, stacks: 5 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 2 },
        Effect::DrawCards { count: 1 },
    ], "Gain 5 Mending. Apply 2 Weakened to ALL. Gain 2 Sorcery Points. Draw 1. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Power];
    d
}

// ── Shadow Sorcerer pool: Terrors synergy ─────────────────────────────

fn dread_bolt_def() -> CardDef {
    let mut d = sorcerer_sc(DREAD_BOLT, "Dread Bolt", "shadow_sorcerer", 1, vec![
        Effect::DealDamage { target: Target::SingleEnemy, amount: 8 },
        Effect::ApplyStatus { target: Target::SingleEnemy, status: StatusType::Frightened, stacks: 2 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 8 damage. Apply 2 Frightened. Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack, CardTag::Skill];
    d
}

fn terrorize_def() -> CardDef {
    let mut d = sorcerer_sc(TERRORIZE, "Terrorize", "shadow_sorcerer", 0, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Frightened, stacks: 1 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Apply 1 Frightened to ALL enemies. Gain 1 Sorcery Point.");
    d.rarity = Rarity::Uncommon;
    d.tags = vec![CardTag::Skill];
    d
}

fn shadow_exploit_def() -> CardDef {
    let mut d = sorcerer_sc(SHADOW_EXPLOIT, "Shadow Exploit", "shadow_sorcerer", 1, vec![
        Effect::DealDamageIfTargetDebuffed { target: Target::SingleEnemy, amount: 10, bonus: 12 },
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 1 },
    ], "Deal 10 damage (22 if debuffed). Gain 1 Sorcery Point.");
    d.tags = vec![CardTag::Attack];
    d
}

fn final_nightmare_def() -> CardDef {
    let mut d = sorcerer_sc(FINAL_NIGHTMARE, "Final Nightmare", "shadow_sorcerer", 2, vec![
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Frightened, stacks: 3 },
        Effect::ApplyStatus { target: Target::AllEnemies, status: StatusType::Weakened, stacks: 3 },
        Effect::DealDamagePerPlayerStatus { target: Target::AllEnemies, status: StatusType::ArcaneCharge, per_stack: 2 },
    ], "Apply 3 Frightened + 3 Weakened to ALL. Deal 2 damage per Sorcery Point to ALL. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Attack];
    d
}

// ── Sorcerer ultimate ─────────────────────────────────────────────────

fn sorcerers_apotheosis_def() -> CardDef {
    let mut d = sorcerer(SORCERERS_APOTHEOSIS, "Sorcerer's Apotheosis", 0, vec![
        Effect::ApplyStatus { target: Target::Player, status: StatusType::ArcaneCharge, stacks: 5 },
        Effect::DrawCards { count: 3 },
        Effect::Heal { amount: 8 },
    ], "Gain 5 Sorcery Points. Draw 3. Heal 8 HP. Innate. Exhaust.");
    d.rarity = Rarity::Legendary;
    d.innate = true;
    d.exhaust = true;
    d.tags = vec![CardTag::Skill, CardTag::Power];
    d
}

// ── Registration ───────────────────────────────────────────────────────

/// All card definitions in the game, keyed by CardId.
pub fn all_card_defs() -> HashMap<CardId, CardDef> {
    let defs = vec![
        // Basics
        strike_def(),
        defend_def(),
        wound_def(),
        // Themed basics
        barbs_def(),
        heavy_strike_def(),
        deft_strike_def(),
        // Class cards
        second_wind_def(),
        energy_surge_def(),
        extra_attack_def(),
        rally_def(),
        // Defense progression
        iron_guard_def(),
        shield_bash_def(),
        brace_def(),
        unbreakable_def(),
        // Two-Handed progression
        intimidating_blow_def(),
        reckless_attack_def(),
        sundering_blow_def(),
        reaving_strike_def(),
        // Dueling progression
        measured_strike_def(),
        sizing_up_def(),
        calculated_strike_def(),
        precise_cut_def(),
        read_and_react_def(),
        expose_weakness_def(),
        flurry_of_cuts_def(),
        feint_def(),
        patient_strike_def(),
        perfect_read_def(),
        riposte_def(),
        perfect_rhythm_def(),
        // Shared reward card definitions
        power_strike_def(),
        quick_slash_def(),
        vicious_strike_def(),
        double_strike_def(),
        brace_for_impact_def(),
        hold_the_line_def(),
        taunt_def(),
        focus_def(),
        martial_ascendancy_def(),
        // Capstones
        iron_fortress_def(),
        titans_fury_def(),
        coup_de_grace_def(),
        // New Phase 2 cards
        adrenaline_rush_def(),
        exploit_opening_def(),
        momentum_def(),
        // Race cards
        improvise_def(),
        fae_ancestry_def(),
        blood_price_def(),
        stonewall_def(),
        flash_bang_def(),
        nimble_dodge_def(),
        savage_charge_def(),
        shiv_def(),
        dragon_breath_def(),
        pounce_def(),
        // Two-Handed: Crusher synergy
        brutal_swing_def(),
        savage_presence_def(),
        demolish_def(),
        wrath_of_the_giant_def(),
        executioners_blow_def(),
        // Two-Handed: Berserker synergy
        blood_frenzy_def(),
        raging_blow_def(),
        berserk_roar_def(),
        unleash_fury_def(),
        deathwish_def(),
        // Defense: Bulwark synergy
        fortify_def(),
        entrench_def(),
        stalwart_defense_def(),
        shield_wall_def(),
        aegis_eternal_def(),
        // Defense: Reprisal synergy
        spiked_armor_def(),
        retribution_def(),
        thorned_carapace_def(),
        barbed_bulwark_def(),
        wrath_of_thorns_def(),
        // Barbarian class + auto-grants
        bloodlust_def(),
        raging_strike_def(),
        totem_guard_def(),
        frenzied_slash_def(),
        battle_fury_def(),
        thick_skin_def(),
        primal_surge_def(),
        undying_rage_def(),
        // Berserker: Bloodrage
        blood_offering_def(),
        fury_unleashed_def(),
        bloodbath_def(),
        pain_is_power_def(),
        berserker_deathwish_def(),
        // Berserker: Rampage
        wild_swing_def(),
        berserk_flurry_def(),
        savage_momentum_def(),
        unstoppable_def(),
        rampage_def(),
        // Totem Warrior: Spirit Shield
        spirit_ward_def(),
        ancestral_shield_def(),
        warding_totem_card_def(),
        fortified_rage_def(),
        unbreaking_spirit_def(),
        // Totem Warrior: Ancestral
        war_cry_def(),
        spirit_mend_def(),
        vengeful_ancestors_def(),
        totem_of_renewal_def(),
        ancestors_embrace_def(),
        // Frenzy: Adrenaline
        desperate_strike_def(),
        adrenaline_spike_def(),
        feral_instinct_def(),
        berserkers_trance_card_def(),
        deaths_door_def(),
        // Frenzy: Overwhelm
        flailing_strike_def(),
        frenzy_card_def(),
        whirlwind_fury_def(),
        one_thousand_cuts_def(),
        // Monk class + auto-grants
        inner_peace_def(),
        centered_strike_def(),
        swift_footwork_def(),
        ki_surge_def(),
        meditation_def(),
        // Monk shared pool (level 1)
        ki_focus_def(),
        ki_strike_def(),
        ki_guard_def(),
        // Open Hand: Flurry
        rapid_strikes_def(),
        palm_strike_def(),
        whirlwind_kick_def(),
        hundred_fists_def(),
        endless_barrage_def(),
        // Open Hand: Ki Burst
        focused_strike_def(),
        ki_shield_def(),
        ki_channeling_def(),
        quivering_palm_def(),
        transcendence_def(),
        // Way of Shadow: Pressure Point
        nerve_strike_def(),
        pressure_point_strike_def(),
        dim_mak_def(),
        crippling_blow_def(),
        weakening_aura_def(),
        death_touch_def(),
        // Way of Shadow: Evasion
        shadow_step_def(),
        deflecting_palm_def(),
        counter_stance_def(),
        phantom_form_def(),
        shadow_kill_def(),
        // Iron Fist: Stance Flow
        stance_shift_def(),
        tigers_fury_def(),
        cranes_wing_def(),
        flowing_water_def(),
        monk_dragons_breath_def(),
        perfect_harmony_def(),
        // Iron Fist: Iron Will
        iron_skin_def(),
        ki_barrier_def(),
        stone_body_def(),
        diamond_soul_def(),
        fist_of_north_star_def(),
        // Warlock class + shared
        eldritch_blast_def(),
        hex_def(),
        dark_bargain_def(),
        maledict_def(),
        pact_barrier_def(),
        siphon_bolt_def(),
        // Infernal: Infernal Burn
        hellbrand_def(),
        cinder_feast_def(),
        balefire_cataclysm_def(),
        // Infernal: Infernal Sustain
        blood_tithe_def(),
        feast_of_cinders_def(),
        // Hexblade: Hexblade Duel
        pact_blade_def(),
        soul_parry_def(),
        dusk_duel_def(),
        // Hexblade: Hexblade Curse
        brand_the_weak_def(),
        black_oath_def(),
        // Void: Void Debuff
        void_gaze_def(),
        starless_whisper_def(),
        entropy_field_def(),
        // Void: Void Annihilation
        oblivion_tide_def(),
        cosmic_extinction_def(),
        // Paladin shared cards
        holy_strike_def(),
        lay_on_hands_def(),
        prayer_of_valor_def(),
        consecration_def(),
        divine_bulwark_def(),
        // Paladin: Devotion
        warding_slash_def(),
        shield_of_faith_def(),
        bastion_drive_def(),
        holy_bastion_def(),
        // Paladin: Vengeance
        avenging_strike_def(),
        blade_of_wrath_def(),
        divine_judgment_def(),
        zealous_rush_def(),
        // Paladin: Radiance
        radiant_burst_def(),
        beacon_of_light_def(),
        solar_flare_def(),
        dawns_blessing_def(),
        // Paladin: Capstone
        avatar_of_radiance_def(),
        // Rogue class + auto-grants
        tricks_of_the_trade_def(),
        sneak_attack_def(),
        cunning_action_def(),
        rogue_evasion_def(),
        preparation_def(),
        // Assassin synergy
        assassinate_def(),
        rogue_shadow_step_def(),
        death_mark_def(),
        killing_blow_def(),
        // Pirate synergy
        cutlass_flurry_def(),
        grappling_hook_def(),
        dirty_fighting_def(),
        broadside_def(),
        // Trickster synergy
        smoke_bomb_def(),
        misdirection_def(),
        fan_of_knives_def(),
        ace_in_the_hole_def(),
        // Wizard shared
        fire_bolt_def(),
        mage_armor_def(),
        arcane_intellect_def(),
        shield_spell_def(),
        // Wizard themed basics
        ember_bolt_def(),
        arcane_ward_def(),
        prescience_def(),
        // Evocation
        fireball_def(),
        scorching_ray_def(),
        lightning_bolt_def(),
        flame_shield_def(),
        // Abjuration
        counterspell_def(),
        ward_def(),
        dispel_def(),
        globe_of_protection_def(),
        // Divination
        foresight_def(),
        scrying_def(),
        portent_def(),
        time_stop_def(),
        // Wizard capstone
        wish_def(),
        // Background cards
        battle_discipline_def(),
        studied_analysis_def(),
        minor_blessing_def(),
        dirty_trick_def(),
        commanding_presence_def(),
        survival_instinct_def(),
        improvised_weapon_def(),
        sea_legs_def(),
        backstab_def(),
        inner_focus_def(),
        resourceful_def(),
        dazzle_def(),
        // Druid: form-entry cantrips
        wild_shape_bear_def(),
        wild_shape_eagle_def(),
        wild_shape_wolf_def(),
        // Druid: shared
        natures_wrath_def(),
        bark_skin_def(),
        rejuvenation_def(),
        // Druid: Bear synergy
        bear_maul_def(),
        thick_hide_def(),
        ursine_charge_def(),
        primal_roar_def(),
        // Druid: Eagle synergy
        eagle_dive_def(),
        swooping_strike_def(),
        wind_rider_def(),
        tempest_talons_def(),
        // Druid: Wolf synergy
        pack_tactics_def(),
        howl_def(),
        coordinated_strike_def(),
        alphas_command_def(),
        // Druid: concentration
        moonbeam_def(),
        entangle_def(),
        // Druid: capstone
        archdruid_def(),
        // Sorcerer: Wild Mage starters
        spark_bolt_def(),
        channel_def(),
        surge_wave_def(),
        arcane_shield_def(),
        wild_surge_def(),
        // Sorcerer: Draconic starters
        claw_strike_def(),
        scale_ward_def(),
        draconic_focus_def(),
        dragons_resilience_def(),
        draconic_bolt_def(),
        // Sorcerer: Shadow Sorcerer starters
        shadow_bolt_def(),
        dark_shroud_def(),
        siphon_def(),
        night_veil_def(),
        umbral_strike_def(),
        // Sorcerer: shared pool
        arcane_pulse_def(),
        mana_infusion_def(),
        spell_surge_def(),
        overload_def(),
        // Sorcerer: Wild Mage pool (Surge)
        volatile_burst_def(),
        chain_lightning_def(),
        surge_nova_def(),
        wild_discharge_def(),
        sorcerer_cataclysm_def(),
        // Sorcerer: Wild Mage pool (Chaos)
        chaos_bolt_def(),
        unraveling_hex_def(),
        entropy_cascade_def(),
        sorcerer_pandemonium_def(),
        // Sorcerer: Draconic pool (Heritage)
        dragon_scale_def(),
        ancestral_resilience_def(),
        scaled_rebuke_def(),
        primordial_resilience_def(),
        // Sorcerer: Draconic pool (Wrath)
        sorcerer_dragons_breath_def(),
        draconic_surge_def(),
        ancient_power_def(),
        dragonfire_ascension_def(),
        // Sorcerer: Shadow Sorcerer pool (Drain)
        shadow_tap_def(),
        necrotic_siphon_def(),
        life_drain_def(),
        void_feast_def(),
        eternal_hunger_def(),
        // Sorcerer: Shadow Sorcerer pool (Terrors)
        dread_bolt_def(),
        terrorize_def(),
        shadow_exploit_def(),
        final_nightmare_def(),
        // Sorcerer: ultimate
        sorcerers_apotheosis_def(),
    ];
    defs.into_iter().map(|d| (d.id.clone(), d)).collect()
}

// ── Unified progression table ───────────────────────────────────────────

/// How a card is obtained during a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acquisition {
    /// Given directly when the unlock level is reached.
    AutoGrant,
    /// Appears in the pick-one-of-three reward pool.
    Pool,
    /// Both: auto-granted AND available in the reward pool.
    Both,
}

/// A single entry in the progression table.
#[derive(Debug, Clone)]
pub struct ProgressionEntry {
    pub card_id: &'static str,
    pub subclasses: &'static [&'static str],
    pub unlock_level: u32,
    pub acquisition: Acquisition,
    /// Optional synergy group tag for balance-testing filters (e.g. "marked", "tempo").
    /// `None` means neutral/shared — always included regardless of synergy filter.
    pub synergy: Option<&'static str>,
    /// Number of copies to grant (for AutoGrant/Both). Defaults to 1.
    pub count: u32,
}

const ALL_FIGHTER: &[&str] = &["defense", "two_handed", "dueling"];

/// The single source of truth for all card acquisition during a run.
///
/// Every card a player can obtain (beyond the starter deck) is listed here.
/// Each entry specifies who can get it, when it unlocks, and how it's obtained.
///
/// See docs/progression.md for the design rationale.
/// Subclasses that see each shared pool card.
/// Dueling sees: Quick Slash, Double Strike, Focus, Brace for Impact (4 of 6).
/// Defense and Two-Handed see all 6 for now (will be curated when those subclasses are reworked).
const SHARED_ALL: &[&str] = &["defense", "two_handed", "dueling"];
const SHARED_NO_DUELING: &[&str] = &["defense", "two_handed"];

const PROGRESSION_TABLE: &[ProgressionEntry] = &[
    // ── Level 1: shared fighter pool (available from the start) ──────
    // Themed basics — subclass-exclusive pool cards (defense, two-handed only).
    ProgressionEntry { card_id: BARBS,            subclasses: &["defense"],      unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: HEAVY_STRIKE,     subclasses: &["two_handed"],   unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    // Shared fighter pool (dueling sees 4 of 6).
    ProgressionEntry { card_id: QUICK_SLASH,      subclasses: SHARED_ALL,        unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: DOUBLE_STRIKE,    subclasses: SHARED_ALL,        unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: BRACE_FOR_IMPACT, subclasses: SHARED_ALL,        unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: FOCUS,            subclasses: SHARED_ALL,        unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: HOLD_THE_LINE,    subclasses: SHARED_NO_DUELING, unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: TAUNT,            subclasses: SHARED_NO_DUELING, unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Shared auto-grants (class-wide utility cards) ────────────────
    ProgressionEntry { card_id: ENERGY_SURGE,     subclasses: ALL_FIGHTER,       unlock_level: 2,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: EXTRA_ATTACK,     subclasses: ALL_FIGHTER,       unlock_level: 4,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: ADRENALINE_RUSH,  subclasses: ALL_FIGHTER,       unlock_level: 6,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: RALLY,            subclasses: ALL_FIGHTER,       unlock_level: 7,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Level 3: dueling subclass specialization begins ──────────────
    ProgressionEntry { card_id: DEFT_STRIKE,       subclasses: &["dueling"],     unlock_level: 3,  acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: MEASURED_STRIKE,   subclasses: &["dueling"],     unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("marked"), count: 1 },
    ProgressionEntry { card_id: FEINT,             subclasses: &["dueling"],     unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("tempo"), count: 1 },

    // ── Subclass progression auto-grants (defense, two-handed) ───────
    // Defense auto-grants
    ProgressionEntry { card_id: IRON_GUARD,        subclasses: &["defense"],     unlock_level: 3,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: SHIELD_BASH,       subclasses: &["defense"],     unlock_level: 6,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: BRACE,             subclasses: &["defense"],     unlock_level: 9,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: UNBREAKABLE,       subclasses: &["defense"],     unlock_level: 11, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    // Defense: Bulwark synergy pool
    ProgressionEntry { card_id: FORTIFY,           subclasses: &["defense"],     unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("bulwark"), count: 1 },
    ProgressionEntry { card_id: ENTRENCH,          subclasses: &["defense"],     unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("bulwark"), count: 1 },
    ProgressionEntry { card_id: STALWART_DEFENSE,  subclasses: &["defense"],     unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("bulwark"), count: 1 },
    ProgressionEntry { card_id: SHIELD_WALL,       subclasses: &["defense"],     unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("bulwark"), count: 1 },
    ProgressionEntry { card_id: AEGIS_ETERNAL,     subclasses: &["defense"],     unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("bulwark"), count: 1 },
    // Defense: Reprisal synergy pool
    ProgressionEntry { card_id: SPIKED_ARMOR,      subclasses: &["defense"],     unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("reprisal"), count: 1 },
    ProgressionEntry { card_id: RETRIBUTION,       subclasses: &["defense"],     unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("reprisal"), count: 1 },
    ProgressionEntry { card_id: THORNED_CARAPACE,  subclasses: &["defense"],     unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("reprisal"), count: 1 },
    ProgressionEntry { card_id: BARBED_BULWARK,    subclasses: &["defense"],     unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("reprisal"), count: 1 },
    ProgressionEntry { card_id: WRATH_OF_THORNS,   subclasses: &["defense"],     unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("reprisal"), count: 1 },
    // Two-handed auto-grants
    ProgressionEntry { card_id: INTIMIDATING_BLOW, subclasses: &["two_handed"],  unlock_level: 3,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: RECKLESS_ATTACK,   subclasses: &["two_handed"],  unlock_level: 6,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: SUNDERING_BLOW,    subclasses: &["two_handed"],  unlock_level: 9,  acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: REAVING_STRIKE,    subclasses: &["two_handed"],  unlock_level: 11, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    // Two-handed: Crusher synergy pool
    ProgressionEntry { card_id: BRUTAL_SWING,       subclasses: &["two_handed"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("crusher"), count: 1 },
    ProgressionEntry { card_id: SAVAGE_PRESENCE,    subclasses: &["two_handed"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("crusher"), count: 1 },
    ProgressionEntry { card_id: DEMOLISH,           subclasses: &["two_handed"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("crusher"), count: 1 },
    ProgressionEntry { card_id: WRATH_OF_THE_GIANT, subclasses: &["two_handed"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("crusher"), count: 1 },
    ProgressionEntry { card_id: EXECUTIONERS_BLOW,  subclasses: &["two_handed"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("crusher"), count: 1 },
    // Two-handed: Berserker synergy pool
    ProgressionEntry { card_id: BLOOD_FRENZY,       subclasses: &["two_handed"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("berserker"), count: 1 },
    ProgressionEntry { card_id: RAGING_BLOW,        subclasses: &["two_handed"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("berserker"), count: 1 },
    ProgressionEntry { card_id: BERSERK_ROAR,       subclasses: &["two_handed"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("berserker"), count: 1 },
    ProgressionEntry { card_id: UNLEASH_FURY,       subclasses: &["two_handed"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("berserker"), count: 1 },
    ProgressionEntry { card_id: DEATHWISH,          subclasses: &["two_handed"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("berserker"), count: 1 },

    // ── Level 5: dueling pool expansion ──────────────────────────────
    ProgressionEntry { card_id: RIPOSTE,           subclasses: &["dueling"],     unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("marked"), count: 1 },
    ProgressionEntry { card_id: READ_AND_REACT,    subclasses: &["dueling"],     unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("tempo"), count: 1 },

    // ── Level 7: dueling pool expansion ──────────────────────────────
    ProgressionEntry { card_id: FLURRY_OF_CUTS,    subclasses: &["dueling"],     unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("tempo"), count: 1 },

    // ── Level 9: dueling pool expansion ──────────────────────────────
    ProgressionEntry { card_id: EXPLOIT_OPENING,   subclasses: &["dueling"],     unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("marked"), count: 1 },
    ProgressionEntry { card_id: MOMENTUM,          subclasses: &["dueling"],     unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("tempo"), count: 1 },

    // ── Level 11: capstone choices ───────────────────────────────────
    ProgressionEntry { card_id: COUP_DE_GRACE,     subclasses: &["dueling"],     unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("marked"), count: 1 },
    ProgressionEntry { card_id: PERFECT_RHYTHM,    subclasses: &["dueling"],     unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("tempo"), count: 1 },

    // ── Level 12 capstones ───────────────────────────────────────────
    ProgressionEntry { card_id: IRON_FORTRESS,     subclasses: &["defense"],     unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: TITANS_FURY,       subclasses: &["two_handed"],  unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: PERFECT_READ,      subclasses: &["dueling"],     unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // BARBARIAN
    // ══════════════════════════════════════════════════════════════════

    // ── Barbarian shared auto-grants ──────────────────────────────────
    ProgressionEntry { card_id: BATTLE_FURY,       subclasses: &["berserker", "totem_warrior", "frenzy"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: THICK_SKIN,        subclasses: &["berserker", "totem_warrior", "frenzy"], unlock_level: 4, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: PRIMAL_SURGE,      subclasses: &["berserker", "totem_warrior", "frenzy"], unlock_level: 6, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: UNDYING_RAGE,      subclasses: &["berserker", "totem_warrior", "frenzy"], unlock_level: 7, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Barbarian shared pool (level 1) ───────────────────────────────
    ProgressionEntry { card_id: RAGING_STRIKE,     subclasses: &["berserker"],      unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: TOTEM_GUARD,       subclasses: &["totem_warrior"],  unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: FRENZIED_SLASH,    subclasses: &["frenzy"],         unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Berserker: Bloodrage synergy pool ──────────────────────────────
    ProgressionEntry { card_id: BLOOD_OFFERING,       subclasses: &["berserker"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("bloodrage"), count: 1 },
    ProgressionEntry { card_id: FURY_UNLEASHED,       subclasses: &["berserker"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("bloodrage"), count: 1 },
    ProgressionEntry { card_id: BLOODBATH,            subclasses: &["berserker"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("bloodrage"), count: 1 },
    ProgressionEntry { card_id: PAIN_IS_POWER,        subclasses: &["berserker"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("bloodrage"), count: 1 },
    ProgressionEntry { card_id: BERSERKER_DEATHWISH,  subclasses: &["berserker"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("bloodrage"), count: 1 },

    // ── Berserker: Rampage synergy pool ────────────────────────────────
    ProgressionEntry { card_id: WILD_SWING,           subclasses: &["berserker"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("rampage"), count: 1 },
    ProgressionEntry { card_id: BERSERK_FLURRY,       subclasses: &["berserker"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("rampage"), count: 1 },
    ProgressionEntry { card_id: SAVAGE_MOMENTUM,      subclasses: &["berserker"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("rampage"), count: 1 },
    ProgressionEntry { card_id: UNSTOPPABLE,          subclasses: &["berserker"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("rampage"), count: 1 },
    ProgressionEntry { card_id: RAMPAGE,              subclasses: &["berserker"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("rampage"), count: 1 },

    // ── Totem Warrior: Spirit Shield synergy pool ──────────────────────
    ProgressionEntry { card_id: SPIRIT_WARD,          subclasses: &["totem_warrior"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("spirit_shield"), count: 1 },
    ProgressionEntry { card_id: ANCESTRAL_SHIELD,     subclasses: &["totem_warrior"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("spirit_shield"), count: 1 },
    ProgressionEntry { card_id: FORTIFIED_RAGE,       subclasses: &["totem_warrior"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("spirit_shield"), count: 1 },
    ProgressionEntry { card_id: WARDING_TOTEM_CARD,   subclasses: &["totem_warrior"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("spirit_shield"), count: 1 },
    ProgressionEntry { card_id: UNBREAKING_SPIRIT,    subclasses: &["totem_warrior"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("spirit_shield"), count: 1 },

    // ── Totem Warrior: Ancestral synergy pool ──────────────────────────
    ProgressionEntry { card_id: WAR_CRY,              subclasses: &["totem_warrior"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("ancestral"), count: 1 },
    ProgressionEntry { card_id: SPIRIT_MEND,          subclasses: &["totem_warrior"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("ancestral"), count: 1 },
    ProgressionEntry { card_id: VENGEFUL_ANCESTORS,   subclasses: &["totem_warrior"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("ancestral"), count: 1 },
    ProgressionEntry { card_id: TOTEM_OF_RENEWAL,     subclasses: &["totem_warrior"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("ancestral"), count: 1 },
    ProgressionEntry { card_id: ANCESTORS_EMBRACE,    subclasses: &["totem_warrior"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("ancestral"), count: 1 },

    // ── Frenzy: Adrenaline synergy pool ────────────────────────────────
    ProgressionEntry { card_id: DESPERATE_STRIKE,     subclasses: &["frenzy"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("adrenaline"), count: 1 },
    ProgressionEntry { card_id: ADRENALINE_SPIKE,     subclasses: &["frenzy"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("adrenaline"), count: 1 },
    ProgressionEntry { card_id: FERAL_INSTINCT,       subclasses: &["frenzy"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("adrenaline"), count: 1 },
    ProgressionEntry { card_id: BERSERKERS_TRANCE_CARD, subclasses: &["frenzy"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("adrenaline"), count: 1 },
    ProgressionEntry { card_id: DEATHS_DOOR,          subclasses: &["frenzy"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("adrenaline"), count: 1 },

    // ── Frenzy: Overwhelm synergy pool ─────────────────────────────────
    ProgressionEntry { card_id: FLAILING_STRIKE,      subclasses: &["frenzy"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("overwhelm"), count: 1 },
    ProgressionEntry { card_id: FRENZY_CARD,          subclasses: &["frenzy"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("overwhelm"), count: 1 },
    ProgressionEntry { card_id: WHIRLWIND_FURY,       subclasses: &["frenzy"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("overwhelm"), count: 1 },
    ProgressionEntry { card_id: ONE_THOUSAND_CUTS,    subclasses: &["frenzy"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("overwhelm"), count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // MONK
    // ══════════════════════════════════════════════════════════════════

    // ── Level 1: shared monk pool (available from the start) ─────────
    ProgressionEntry { card_id: KI_FOCUS,           subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: KI_STRIKE,          subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: KI_GUARD,           subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Monk shared auto-grants ───────────────────────────────────────
    ProgressionEntry { card_id: CENTERED_STRIKE,    subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: SWIFT_FOOTWORK,     subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 4, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: KI_SURGE,           subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 6, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: MEDITATION,         subclasses: &["open_hand", "way_of_shadow", "iron_fist"], unlock_level: 7, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Open Hand ─────────────────────────────────────────────────────
    ProgressionEntry { card_id: RAPID_STRIKES,       subclasses: &["open_hand"], unlock_level: 3,  acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: PALM_STRIKE,         subclasses: &["open_hand"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("flurry"), count: 1 },
    ProgressionEntry { card_id: FOCUSED_STRIKE,      subclasses: &["open_hand"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("ki_burst"), count: 1 },
    ProgressionEntry { card_id: WHIRLWIND_KICK,      subclasses: &["open_hand"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("flurry"), count: 1 },
    ProgressionEntry { card_id: KI_SHIELD,           subclasses: &["open_hand"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("ki_burst"), count: 1 },
    ProgressionEntry { card_id: HUNDRED_FISTS,       subclasses: &["open_hand"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("flurry"), count: 1 },
    ProgressionEntry { card_id: KI_CHANNELING,       subclasses: &["open_hand"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("ki_burst"), count: 1 },
    ProgressionEntry { card_id: ENDLESS_BARRAGE,     subclasses: &["open_hand"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("flurry"), count: 1 },
    ProgressionEntry { card_id: QUIVERING_PALM,      subclasses: &["open_hand"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("ki_burst"), count: 1 },
    ProgressionEntry { card_id: TRANSCENDENCE,       subclasses: &["open_hand"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Way of Shadow ─────────────────────────────────────────────────
    ProgressionEntry { card_id: NERVE_STRIKE,            subclasses: &["way_of_shadow"], unlock_level: 3,  acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: PRESSURE_POINT_STRIKE,   subclasses: &["way_of_shadow"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("pressure_point"), count: 1 },
    ProgressionEntry { card_id: SHADOW_STEP,             subclasses: &["way_of_shadow"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("evasion"), count: 1 },
    ProgressionEntry { card_id: DIM_MAK,                 subclasses: &["way_of_shadow"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("pressure_point"), count: 1 },
    ProgressionEntry { card_id: DEFLECTING_PALM,         subclasses: &["way_of_shadow"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("evasion"), count: 1 },
    ProgressionEntry { card_id: CRIPPLING_BLOW,          subclasses: &["way_of_shadow"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("pressure_point"), count: 1 },
    ProgressionEntry { card_id: COUNTER_STANCE,          subclasses: &["way_of_shadow"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("evasion"), count: 1 },
    ProgressionEntry { card_id: WEAKENING_AURA,          subclasses: &["way_of_shadow"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("pressure_point"), count: 1 },
    ProgressionEntry { card_id: DEATH_TOUCH,             subclasses: &["way_of_shadow"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("pressure_point"), count: 1 },
    ProgressionEntry { card_id: PHANTOM_FORM,            subclasses: &["way_of_shadow"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("evasion"), count: 1 },
    ProgressionEntry { card_id: SHADOW_KILL,             subclasses: &["way_of_shadow"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Iron Fist ─────────────────────────────────────────────────────
    ProgressionEntry { card_id: STANCE_SHIFT,        subclasses: &["iron_fist"], unlock_level: 3,  acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: TIGERS_FURY,         subclasses: &["iron_fist"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("stance_flow"), count: 1 },
    ProgressionEntry { card_id: CRANES_WING,         subclasses: &["iron_fist"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("stance_flow"), count: 1 },
    ProgressionEntry { card_id: IRON_SKIN,           subclasses: &["iron_fist"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("iron_will"), count: 1 },
    ProgressionEntry { card_id: FLOWING_WATER,       subclasses: &["iron_fist"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("stance_flow"), count: 1 },
    ProgressionEntry { card_id: KI_BARRIER,          subclasses: &["iron_fist"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("iron_will"), count: 1 },
    ProgressionEntry { card_id: MONK_DRAGONS_BREATH, subclasses: &["iron_fist"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("stance_flow"), count: 1 },
    ProgressionEntry { card_id: STONE_BODY,          subclasses: &["iron_fist"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("iron_will"), count: 1 },
    ProgressionEntry { card_id: PERFECT_HARMONY,     subclasses: &["iron_fist"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("stance_flow"), count: 1 },
    ProgressionEntry { card_id: DIAMOND_SOUL,        subclasses: &["iron_fist"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("iron_will"), count: 1 },
    ProgressionEntry { card_id: FIST_OF_NORTH_STAR,  subclasses: &["iron_fist"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // WARLOCK
    // ══════════════════════════════════════════════════════════════════

    // ── Level 1: shared warlock pool ─────────────────────────────────
    ProgressionEntry { card_id: PACT_BARRIER,          subclasses: &["infernal", "hexblade", "void"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: SIPHON_BOLT,           subclasses: &["infernal", "hexblade", "void"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: MALEDICT,              subclasses: &["infernal", "hexblade", "void"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Warlock shared auto-grants ───────────────────────────────────
    ProgressionEntry { card_id: DARK_BARGAIN,          subclasses: &["infernal", "hexblade", "void"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Infernal: Infernal Burn synergy pool ─────────────────────────
    ProgressionEntry { card_id: HELLBRAND,             subclasses: &["infernal"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("infernal_burn"), count: 1 },
    ProgressionEntry { card_id: CINDER_FEAST,          subclasses: &["infernal"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("infernal_burn"), count: 1 },
    ProgressionEntry { card_id: BALEFIRE_CATACLYSM,    subclasses: &["infernal"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("infernal_burn"), count: 1 },

    // ── Infernal: Infernal Sustain synergy pool ──────────────────────
    ProgressionEntry { card_id: BLOOD_TITHE,           subclasses: &["infernal"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("infernal_sustain"), count: 1 },
    ProgressionEntry { card_id: FEAST_OF_CINDERS,      subclasses: &["infernal"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("infernal_sustain"), count: 1 },

    // ── Hexblade: Hexblade Duel synergy pool ─────────────────────────
    ProgressionEntry { card_id: PACT_BLADE,            subclasses: &["hexblade"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("hexblade_duel"), count: 1 },
    ProgressionEntry { card_id: SOUL_PARRY,            subclasses: &["hexblade"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("hexblade_duel"), count: 1 },
    ProgressionEntry { card_id: DUSK_DUEL,             subclasses: &["hexblade"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("hexblade_duel"), count: 1 },

    // ── Hexblade: Hexblade Curse synergy pool ────────────────────────
    ProgressionEntry { card_id: BRAND_THE_WEAK,        subclasses: &["hexblade"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("hexblade_curse"), count: 1 },
    ProgressionEntry { card_id: BLACK_OATH,            subclasses: &["hexblade"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("hexblade_curse"), count: 1 },

    // ── Void: Void Debuff synergy pool ───────────────────────────────
    ProgressionEntry { card_id: VOID_GAZE,             subclasses: &["void"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("void_debuff"), count: 1 },
    ProgressionEntry { card_id: STARLESS_WHISPER,      subclasses: &["void"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("void_debuff"), count: 1 },
    ProgressionEntry { card_id: ENTROPY_FIELD,         subclasses: &["void"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("void_debuff"), count: 1 },

    // ── Void: Void Annihilation synergy pool ─────────────────────────
    ProgressionEntry { card_id: OBLIVION_TIDE,         subclasses: &["void"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("void_annihilation"), count: 1 },
    ProgressionEntry { card_id: COSMIC_EXTINCTION,     subclasses: &["void"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("void_annihilation"), count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // PALADIN
    // ══════════════════════════════════════════════════════════════════

    // ── Level 1: shared martial pool (shared with Fighter) ──────────
    ProgressionEntry { card_id: QUICK_SLASH,      subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: DOUBLE_STRIKE,    subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: BRACE_FOR_IMPACT, subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: FOCUS,            subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: HOLD_THE_LINE,    subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: TAUNT,            subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Paladin shared auto-grants ──────────────────────────────────
    ProgressionEntry { card_id: CONSECRATION,     subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: DIVINE_BULWARK,   subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 4, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: LAY_ON_HANDS,     subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 6, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Devotion: devotion_shield synergy pool ──────────────────────
    ProgressionEntry { card_id: WARDING_SLASH,    subclasses: &["devotion"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("devotion_shield"), count: 1 },
    ProgressionEntry { card_id: SHIELD_OF_FAITH,  subclasses: &["devotion"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("devotion_shield"), count: 1 },
    ProgressionEntry { card_id: HOLY_BASTION,     subclasses: &["devotion"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("devotion_shield"), count: 1 },
    // ── Devotion: devotion_armor synergy pool ───────────────────────
    ProgressionEntry { card_id: BASTION_DRIVE,    subclasses: &["devotion"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("devotion_armor"), count: 1 },

    // ── Vengeance: vengeance_smite synergy pool ─────────────────────
    ProgressionEntry { card_id: AVENGING_STRIKE,  subclasses: &["vengeance"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("vengeance_smite"), count: 1 },
    ProgressionEntry { card_id: DIVINE_JUDGMENT,  subclasses: &["vengeance"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("vengeance_smite"), count: 1 },
    // ── Vengeance: vengeance_wrath synergy pool ─────────────────────
    ProgressionEntry { card_id: BLADE_OF_WRATH,   subclasses: &["vengeance"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("vengeance_wrath"), count: 1 },
    ProgressionEntry { card_id: ZEALOUS_RUSH,     subclasses: &["vengeance"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("vengeance_wrath"), count: 1 },

    // ── Radiance: radiance_burst synergy pool ───────────────────────
    ProgressionEntry { card_id: RADIANT_BURST,    subclasses: &["radiance"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("radiance_burst"), count: 1 },
    ProgressionEntry { card_id: SOLAR_FLARE,      subclasses: &["radiance"], unlock_level: 9,  acquisition: Acquisition::Pool, synergy: Some("radiance_burst"), count: 1 },
    // ── Radiance: radiance_heal synergy pool ────────────────────────
    ProgressionEntry { card_id: BEACON_OF_LIGHT,  subclasses: &["radiance"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("radiance_heal"), count: 1 },
    ProgressionEntry { card_id: DAWNS_BLESSING,   subclasses: &["radiance"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("radiance_heal"), count: 1 },

    // ── Paladin capstones ───────────────────────────────────────────
    ProgressionEntry { card_id: AVATAR_OF_RADIANCE, subclasses: &["devotion", "vengeance", "radiance"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // ROGUE
    // ══════════════════════════════════════════════════════════════════

    // ── Rogue shared auto-grants ────────────────────────────────────
    ProgressionEntry { card_id: SNEAK_ATTACK,      subclasses: &["assassin", "pirate", "trickster"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: CUNNING_ACTION,    subclasses: &["assassin", "pirate", "trickster"], unlock_level: 4, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: ROGUE_EVASION,     subclasses: &["assassin", "pirate", "trickster"], unlock_level: 6, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: PREPARATION,       subclasses: &["assassin", "pirate", "trickster"], unlock_level: 7, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Rogue shared martial pool (level 1) ─────────────────────────
    // Rogues share the Martial reward pool with Fighter.
    ProgressionEntry { card_id: QUICK_SLASH,       subclasses: &["assassin", "pirate", "trickster"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: DOUBLE_STRIKE,     subclasses: &["assassin", "pirate", "trickster"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: BRACE_FOR_IMPACT,  subclasses: &["assassin", "pirate", "trickster"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: FOCUS,             subclasses: &["assassin", "pirate", "trickster"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Assassin synergy: assassination ──────────────────────────────
    ProgressionEntry { card_id: ASSASSINATE,       subclasses: &["assassin"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("assassination"), count: 1 },
    ProgressionEntry { card_id: ROGUE_SHADOW_STEP, subclasses: &["assassin"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("shadow"), count: 1 },
    ProgressionEntry { card_id: DEATH_MARK,        subclasses: &["assassin"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("assassination"), count: 1 },
    ProgressionEntry { card_id: KILLING_BLOW,      subclasses: &["assassin"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("shadow"), count: 1 },

    // ── Pirate synergy: pirate_blade / pirate_trick ─────────────────
    ProgressionEntry { card_id: CUTLASS_FLURRY,    subclasses: &["pirate"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("pirate_blade"), count: 1 },
    ProgressionEntry { card_id: GRAPPLING_HOOK,    subclasses: &["pirate"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("pirate_trick"), count: 1 },
    ProgressionEntry { card_id: DIRTY_FIGHTING,    subclasses: &["pirate"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("pirate_blade"), count: 1 },
    ProgressionEntry { card_id: BROADSIDE,         subclasses: &["pirate"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("pirate_trick"), count: 1 },

    // ── Trickster synergy: trickster_smoke / trickster_finesse ──────
    ProgressionEntry { card_id: SMOKE_BOMB,        subclasses: &["trickster"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("trickster_smoke"), count: 1 },
    ProgressionEntry { card_id: MISDIRECTION,      subclasses: &["trickster"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("trickster_finesse"), count: 1 },
    ProgressionEntry { card_id: FAN_OF_KNIVES,     subclasses: &["trickster"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("trickster_smoke"), count: 1 },
    ProgressionEntry { card_id: ACE_IN_THE_HOLE,   subclasses: &["trickster"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("trickster_finesse"), count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // DRUID
    // ══════════════════════════════════════════════════════════════════

    // ── Druid shared pool (level 1) ─────────────────────────────────
    ProgressionEntry { card_id: BARK_SKIN,        subclasses: &["bear", "eagle", "wolf"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: REJUVENATION,     subclasses: &["bear", "eagle", "wolf"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Druid shared auto-grants ────────────────────────────────────
    ProgressionEntry { card_id: MOONBEAM,         subclasses: &["bear", "eagle", "wolf"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: ENTANGLE,         subclasses: &["bear", "eagle", "wolf"], unlock_level: 4, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Bear: bear_tank synergy pool ────────────────────────────────
    ProgressionEntry { card_id: THICK_HIDE,       subclasses: &["bear"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("bear_tank"), count: 1 },
    ProgressionEntry { card_id: PRIMAL_ROAR,      subclasses: &["bear"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("bear_tank"), count: 1 },

    // ── Bear: bear_primal synergy pool ──────────────────────────────
    ProgressionEntry { card_id: BEAR_MAUL,        subclasses: &["bear"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("bear_primal"), count: 1 },
    ProgressionEntry { card_id: URSINE_CHARGE,    subclasses: &["bear"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("bear_primal"), count: 1 },

    // ── Eagle: eagle_swoop synergy pool ─────────────────────────────
    ProgressionEntry { card_id: EAGLE_DIVE,       subclasses: &["eagle"], unlock_level: 3, acquisition: Acquisition::Pool, synergy: Some("eagle_swoop"), count: 1 },
    ProgressionEntry { card_id: TEMPEST_TALONS,   subclasses: &["eagle"], unlock_level: 5, acquisition: Acquisition::Pool, synergy: Some("eagle_swoop"), count: 1 },

    // ── Eagle: eagle_wind synergy pool ──────────────────────────────
    ProgressionEntry { card_id: SWOOPING_STRIKE,  subclasses: &["eagle"], unlock_level: 3, acquisition: Acquisition::Pool, synergy: Some("eagle_wind"), count: 1 },
    ProgressionEntry { card_id: WIND_RIDER,       subclasses: &["eagle"], unlock_level: 5, acquisition: Acquisition::Pool, synergy: Some("eagle_wind"), count: 1 },

    // ── Wolf: wolf_pack synergy pool ────────────────────────────────
    ProgressionEntry { card_id: PACK_TACTICS,     subclasses: &["wolf"], unlock_level: 3, acquisition: Acquisition::Pool, synergy: Some("wolf_pack"), count: 1 },
    ProgressionEntry { card_id: ALPHAS_COMMAND,   subclasses: &["wolf"], unlock_level: 5, acquisition: Acquisition::Pool, synergy: Some("wolf_pack"), count: 1 },

    // ── Wolf: wolf_hunt synergy pool ────────────────────────────────
    ProgressionEntry { card_id: HOWL,             subclasses: &["wolf"], unlock_level: 3, acquisition: Acquisition::Pool, synergy: Some("wolf_hunt"), count: 1 },
    ProgressionEntry { card_id: COORDINATED_STRIKE, subclasses: &["wolf"], unlock_level: 5, acquisition: Acquisition::Pool, synergy: Some("wolf_hunt"), count: 1 },

    // ── Druid capstone ──────────────────────────────────────────────
    ProgressionEntry { card_id: ARCHDRUID,        subclasses: &["bear", "eagle", "wolf"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // WIZARD
    // ══════════════════════════════════════════════════════════════════

    // ── Wizard shared auto-grants ───────────────────────────────────────
    ProgressionEntry { card_id: MAGE_ARMOR,         subclasses: &["evocation", "abjuration", "divination"], unlock_level: 2, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
    ProgressionEntry { card_id: SHIELD_SPELL,       subclasses: &["evocation", "abjuration", "divination"], unlock_level: 4, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ── Wizard shared pool (level 1) ────────────────────────────────────
    ProgressionEntry { card_id: FIRE_BOLT,          subclasses: &["evocation", "abjuration", "divination"], unlock_level: 1, acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Evocation ───────────────────────────────────────────────────────
    ProgressionEntry { card_id: EMBER_BOLT,         subclasses: &["evocation"], unlock_level: 3, acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: SCORCHING_RAY,      subclasses: &["evocation"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("evocation_blast"), count: 1 },
    ProgressionEntry { card_id: FLAME_SHIELD,       subclasses: &["evocation"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("evocation_burn"), count: 1 },
    ProgressionEntry { card_id: FIREBALL,           subclasses: &["evocation"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("evocation_blast"), count: 1 },
    ProgressionEntry { card_id: LIGHTNING_BOLT,     subclasses: &["evocation"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("evocation_blast"), count: 1 },

    // ── Abjuration ──────────────────────────────────────────────────────
    ProgressionEntry { card_id: ARCANE_WARD,        subclasses: &["abjuration"], unlock_level: 3, acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: WARD,               subclasses: &["abjuration"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("abjuration_ward"), count: 1 },
    ProgressionEntry { card_id: DISPEL,             subclasses: &["abjuration"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("abjuration_counter"), count: 1 },
    ProgressionEntry { card_id: COUNTERSPELL,       subclasses: &["abjuration"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("abjuration_counter"), count: 1 },
    ProgressionEntry { card_id: GLOBE_OF_PROTECTION, subclasses: &["abjuration"], unlock_level: 7, acquisition: Acquisition::Pool, synergy: Some("abjuration_ward"), count: 1 },

    // ── Divination ──────────────────────────────────────────────────────
    ProgressionEntry { card_id: PRESCIENCE,         subclasses: &["divination"], unlock_level: 3, acquisition: Acquisition::AutoGrant, synergy: None, count: 2 },
    ProgressionEntry { card_id: SCRYING,            subclasses: &["divination"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("divination_sight"), count: 1 },
    ProgressionEntry { card_id: PORTENT,            subclasses: &["divination"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("divination_control"), count: 1 },
    ProgressionEntry { card_id: FORESIGHT,          subclasses: &["divination"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("divination_sight"), count: 1 },
    ProgressionEntry { card_id: TIME_STOP,          subclasses: &["divination"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("divination_control"), count: 1 },

    // ── Wizard capstone ─────────────────────────────────────────────────
    ProgressionEntry { card_id: WISH,               subclasses: &["evocation", "abjuration", "divination"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },

    // ══════════════════════════════════════════════════════════════════
    // SORCERER
    // ══════════════════════════════════════════════════════════════════

    // ── Sorcerer shared pool (all subclasses) ─────────────────────────
    ProgressionEntry { card_id: ARCANE_PULSE,       subclasses: &["wild_mage", "draconic", "shadow_sorcerer"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: SPELL_SURGE,        subclasses: &["wild_mage", "draconic", "shadow_sorcerer"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: MANA_INFUSION,      subclasses: &["wild_mage", "draconic", "shadow_sorcerer"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: None, count: 1 },
    ProgressionEntry { card_id: OVERLOAD,           subclasses: &["wild_mage", "draconic", "shadow_sorcerer"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: None, count: 1 },

    // ── Wild Mage: Surge synergy pool ─────────────────────────────────
    ProgressionEntry { card_id: VOLATILE_BURST,     subclasses: &["wild_mage"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("surge"), count: 1 },
    ProgressionEntry { card_id: CHAIN_LIGHTNING,    subclasses: &["wild_mage"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("surge"), count: 1 },
    ProgressionEntry { card_id: WILD_DISCHARGE,     subclasses: &["wild_mage"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("surge"), count: 1 },
    ProgressionEntry { card_id: SURGE_NOVA,         subclasses: &["wild_mage"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("surge"), count: 1 },
    ProgressionEntry { card_id: SORCERER_CATACLYSM, subclasses: &["wild_mage"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("surge"), count: 1 },

    // ── Wild Mage: Chaos synergy pool ─────────────────────────────────
    ProgressionEntry { card_id: CHAOS_BOLT,         subclasses: &["wild_mage"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("chaos"), count: 1 },
    ProgressionEntry { card_id: UNRAVELING_HEX,     subclasses: &["wild_mage"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("chaos"), count: 1 },
    ProgressionEntry { card_id: ENTROPY_CASCADE,    subclasses: &["wild_mage"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("chaos"), count: 1 },
    ProgressionEntry { card_id: SORCERER_PANDEMONIUM, subclasses: &["wild_mage"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("chaos"), count: 1 },

    // ── Draconic: Heritage synergy pool ───────────────────────────────
    ProgressionEntry { card_id: DRAGON_SCALE,       subclasses: &["draconic"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("heritage"), count: 1 },
    ProgressionEntry { card_id: ANCESTRAL_RESILIENCE, subclasses: &["draconic"], unlock_level: 3, acquisition: Acquisition::Pool, synergy: Some("heritage"), count: 1 },
    ProgressionEntry { card_id: SCALED_REBUKE,      subclasses: &["draconic"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("heritage"), count: 1 },
    ProgressionEntry { card_id: PRIMORDIAL_RESILIENCE, subclasses: &["draconic"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("heritage"), count: 1 },

    // ── Draconic: Wrath synergy pool ──────────────────────────────────
    ProgressionEntry { card_id: SORCERER_DRAGONS_BREATH, subclasses: &["draconic"], unlock_level: 3, acquisition: Acquisition::Pool, synergy: Some("wrath"), count: 1 },
    ProgressionEntry { card_id: DRACONIC_SURGE,     subclasses: &["draconic"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("wrath"), count: 1 },
    ProgressionEntry { card_id: ANCIENT_POWER,      subclasses: &["draconic"], unlock_level: 7,  acquisition: Acquisition::Pool, synergy: Some("wrath"), count: 1 },
    ProgressionEntry { card_id: DRAGONFIRE_ASCENSION, subclasses: &["draconic"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("wrath"), count: 1 },

    // ── Shadow Sorcerer: Drain synergy pool ───────────────────────────
    ProgressionEntry { card_id: SHADOW_TAP,         subclasses: &["shadow_sorcerer"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("drain"), count: 1 },
    ProgressionEntry { card_id: NECROTIC_SIPHON,    subclasses: &["shadow_sorcerer"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("drain"), count: 1 },
    ProgressionEntry { card_id: LIFE_DRAIN,         subclasses: &["shadow_sorcerer"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("drain"), count: 1 },
    ProgressionEntry { card_id: VOID_FEAST,         subclasses: &["shadow_sorcerer"], unlock_level: 5,  acquisition: Acquisition::Pool, synergy: Some("drain"), count: 1 },
    ProgressionEntry { card_id: ETERNAL_HUNGER,     subclasses: &["shadow_sorcerer"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("drain"), count: 1 },

    // ── Shadow Sorcerer: Terrors synergy pool ─────────────────────────
    ProgressionEntry { card_id: DREAD_BOLT,         subclasses: &["shadow_sorcerer"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("terrors"), count: 1 },
    ProgressionEntry { card_id: SHADOW_EXPLOIT,     subclasses: &["shadow_sorcerer"], unlock_level: 1,  acquisition: Acquisition::Pool, synergy: Some("terrors"), count: 1 },
    ProgressionEntry { card_id: TERRORIZE,          subclasses: &["shadow_sorcerer"], unlock_level: 3,  acquisition: Acquisition::Pool, synergy: Some("terrors"), count: 1 },
    ProgressionEntry { card_id: FINAL_NIGHTMARE,    subclasses: &["shadow_sorcerer"], unlock_level: 11, acquisition: Acquisition::Pool, synergy: Some("terrors"), count: 1 },

    // ── Sorcerer ultimate ─────────────────────────────────────────────
    ProgressionEntry { card_id: SORCERERS_APOTHEOSIS, subclasses: &["wild_mage", "draconic", "shadow_sorcerer"], unlock_level: 12, acquisition: Acquisition::AutoGrant, synergy: None, count: 1 },
];

/// Check whether a progression entry passes a synergy filter.
/// `None` filter means everything passes. `Some("marked")` means only entries
/// with `synergy == None` (neutral/shared) or `synergy == Some("marked")` pass.
fn passes_synergy_filter(entry: &ProgressionEntry, filter: Option<&str>) -> bool {
    match (filter, entry.synergy) {
        (None, _) => true,                          // no filter — everything passes
        (_, None) => true,                           // neutral card — always passes
        (Some(f), Some(s)) => f == s,                // must match
    }
}

/// Look up which subclass owns a synergy group.
///
/// Each synergy group belongs to exactly one subclass.  Returns `None` for
/// unknown groups.  This is the authoritative mapping — callers can use it
/// to infer `--subclass` from `--synergy-group`.
pub fn subclass_for_synergy_group(group: &str) -> Option<&'static str> {
    // Scan the progression table for the first entry tagged with this group.
    PROGRESSION_TABLE
        .iter()
        .find(|e| e.synergy == Some(group))
        .and_then(|e| e.subclasses.first().copied())
}

/// Return all Pool/Both cards available to a subclass at a given level.
pub fn reward_pool_at_level(subclass: &str, level: u32) -> Vec<CardId> {
    reward_pool_at_level_filtered(subclass, level, None)
}

/// Return all Pool/Both cards available to a subclass at a given level,
/// optionally filtered by synergy group.
pub fn reward_pool_at_level_filtered(subclass: &str, level: u32, synergy_filter: Option<&str>) -> Vec<CardId> {
    let mut pool: Vec<CardId> = PROGRESSION_TABLE
        .iter()
        .filter(|e| {
            e.unlock_level <= level
                && (e.acquisition == Acquisition::Pool || e.acquisition == Acquisition::Both)
                && e.subclasses.contains(&subclass)
                && passes_synergy_filter(e, synergy_filter)
        })
        .map(|e| e.card_id.into())
        .collect();
    if OP_TEST_CARD_ENABLED {
        pool.push(MARTIAL_ASCENDANCY.into());
    }
    pool
}

/// Return all AutoGrant/Both cards for a subclass at exactly this level.
pub fn auto_grants_at_level(subclass: &str, level: u32) -> Vec<CardId> {
    auto_grants_at_level_filtered(subclass, level, None)
}

/// Return all AutoGrant/Both cards for a subclass at exactly this level,
/// optionally filtered by synergy group.
pub fn auto_grants_at_level_filtered(subclass: &str, level: u32, synergy_filter: Option<&str>) -> Vec<CardId> {
    let mut grants = Vec::new();
    for e in PROGRESSION_TABLE {
        if e.unlock_level == level
            && (e.acquisition == Acquisition::AutoGrant || e.acquisition == Acquisition::Both)
            && e.subclasses.contains(&subclass)
            && passes_synergy_filter(e, synergy_filter)
        {
            for _ in 0..e.count {
                grants.push(e.card_id.into());
            }
        }
    }
    grants
}

/// All card IDs that can appear in a run for a given subclass and synergy filter.
/// This is the union of: starter deck cards + all progression cards that pass the
/// filter + basics (strike, defend, wound). Used to build the dynamic CARD_VOCAB.
pub fn active_card_ids_for_run(subclass: &str, synergy_filter: Option<&str>, race: &str, background: &str) -> Vec<CardId> {
    use decker_engine::card_ids::{STRIKE, DEFEND, WOUND};

    let mut ids: Vec<CardId> = Vec::new();

    // Basics (always present).
    ids.push(STRIKE.into());
    ids.push(DEFEND.into());
    ids.push(WOUND.into());

    // Starter deck cards (includes race + background cards).
    let starter = crate::starter_decks::fighter_starter_deck(subclass, race, background);
    for card in &starter {
        ids.push(card.def_id.clone());
    }

    // All progression cards that pass the filter for this subclass.
    for entry in PROGRESSION_TABLE {
        if entry.subclasses.contains(&subclass) && passes_synergy_filter(entry, synergy_filter) {
            ids.push(entry.card_id.into());
        }
    }

    if OP_TEST_CARD_ENABLED {
        ids.push(MARTIAL_ASCENDANCY.into());
    }

    ids.sort();
    ids.dedup();
    ids
}

/// All card IDs in the progression table (union across all subclasses/levels).
/// Used for content hashing.
pub fn all_progression_card_ids() -> Vec<CardId> {
    let mut ids: Vec<CardId> = PROGRESSION_TABLE
        .iter()
        .map(|e| e.card_id.into())
        .collect();
    ids.sort();
    ids.dedup();
    if OP_TEST_CARD_ENABLED {
        ids.push(MARTIAL_ASCENDANCY.into());
        ids.sort();
    }
    ids
}

/// Generate reward options from the pool.
pub fn generate_rewards(
    pool: &[CardId],
    count: usize,
    rng: &mut decker_engine::rng::GameRng,
) -> Vec<CardId> {
    if pool.is_empty() {
        return vec![];
    }
    let mut rewards = Vec::new();
    for _ in 0..count {
        let idx = rng.range(0, (pool.len() as i32) - 1) as usize;
        rewards.push(pool[idx].clone());
    }
    rewards
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_defs_non_empty() {
        let defs = all_card_defs();
        assert!(!defs.is_empty());
    }

    #[test]
    fn expected_card_count() {
        let defs = all_card_defs();
        // 3 basics + 4 themed (incl deft_strike) + 4 class + 6 dueling existing
        // + 5 dueling new (feint, patient_strike, perfect_read, riposte, perfect_rhythm)
        // + 8 defense/two_handed progression + 7 shared reward (excl power_strike/vicious_strike still in defs)
        // + 9 shared reward + 3 capstones = 42
        // Detailed: strike, defend, wound, barbs, heavy_strike, measured_strike, deft_strike,
        //   second_wind, energy_surge, extra_attack, rally,
        //   iron_guard, shield_bash, brace, unbreakable,
        //   intimidating_blow, reckless_attack, sundering_blow, reaving_strike,
        //   sizing_up, calculated_strike, precise_cut, read_and_react, expose_weakness, flurry_of_cuts,
        //   feint, patient_strike, perfect_read, riposte, perfect_rhythm,
        //   power_strike, quick_slash, vicious_strike, double_strike, brace_for_impact,
        //   hold_the_line, taunt, focus, martial_ascendancy,
        //   iron_fortress, titans_fury, coup_de_grace
        // 42 original + 3 new Phase 2 cards + 10 race cards + 12 background cards = 67
        // 164 previous + 21 warlock + 18 paladin + 17 rogue + 21 druid + 20 wizard = 261
        // 261 + 46 sorcerer (15 starters + 4 shared + 9 surge + 4 chaos + 4 heritage + 4 wrath + 5 drain + 4 terrors + 1 ultimate) = 307
        assert_eq!(defs.len(), 307);
    }

    #[test]
    fn all_progression_cards_exist_in_defs() {
        let defs = all_card_defs();
        for entry in PROGRESSION_TABLE {
            assert!(
                defs.contains_key(entry.card_id),
                "progression table references unknown card: {}",
                entry.card_id
            );
        }
    }

    #[test]
    fn reward_pool_at_level_1_per_subclass() {
        let op = if OP_TEST_CARD_ENABLED { 1 } else { 0 };
        // Defense: barbs + 6 shared = 7
        assert_eq!(reward_pool_at_level("defense", 1).len(), 7 + op,
            "level 1 reward pool for defense");
        // Two-handed: heavy_strike + 6 shared = 7
        assert_eq!(reward_pool_at_level("two_handed", 1).len(), 7 + op,
            "level 1 reward pool for two_handed");
        // Dueling: quick_slash + double_strike + brace_for_impact + focus = 4
        assert_eq!(reward_pool_at_level("dueling", 1).len(), 4 + op,
            "level 1 reward pool for dueling");
    }

    #[test]
    fn reward_pool_themed_basics_are_subclass_exclusive() {
        let defense_pool = reward_pool_at_level("defense", 1);
        let dueling_pool = reward_pool_at_level("dueling", 1);
        let two_handed_pool = reward_pool_at_level("two_handed", 1);

        assert!(defense_pool.contains(&BARBS.into()));
        assert!(!defense_pool.contains(&HEAVY_STRIKE.into()));

        // Dueling has no subclass-specific L1 pool cards (specialization starts at L3)
        assert!(!dueling_pool.contains(&BARBS.into()));
        assert!(!dueling_pool.contains(&HEAVY_STRIKE.into()));

        assert!(two_handed_pool.contains(&HEAVY_STRIKE.into()));
        assert!(!two_handed_pool.contains(&BARBS.into()));
    }

    #[test]
    fn reward_pool_grows_monotonically() {
        for sc in &["defense", "two_handed", "dueling"] {
            let mut prev_len = 0;
            for level in 1..=12 {
                let pool = reward_pool_at_level(sc, level);
                assert!(
                    pool.len() >= prev_len,
                    "pool for {} shrank from level {} to {}",
                    sc, level - 1, level
                );
                prev_len = pool.len();
            }
        }
    }

    #[test]
    fn auto_grants_at_expected_levels() {
        // Defense and Two-handed keep auto-grants at levels 3, 6, 9, 11, 12.
        for sc in &["defense", "two_handed"] {
            for level in [3, 6, 9, 11, 12] {
                let grants = auto_grants_at_level(sc, level);
                assert!(
                    !grants.is_empty(),
                    "{} should have auto-grants at level {}",
                    sc, level
                );
            }
        }

        // Dueling gets Deft Strike ×2 at level 3
        let dueling_3 = auto_grants_at_level("dueling", 3);
        assert_eq!(dueling_3.len(), 2, "dueling level 3: Deft Strike ×2");
        assert!(dueling_3.iter().all(|id| id == DEFT_STRIKE));

        // Dueling level 12 ultimate
        assert_eq!(auto_grants_at_level("dueling", 12).len(), 1, "dueling level 12 ultimate");

        // Shared utility auto-grants for all subclasses
        for sc in &["defense", "two_handed", "dueling"] {
            assert!(!auto_grants_at_level(sc, 2).is_empty(), "level 2 Energy Surge");
            assert!(!auto_grants_at_level(sc, 4).is_empty(), "level 4 Extra Attack");
            assert!(!auto_grants_at_level(sc, 6).is_empty(), "level 6 Adrenaline Rush");
            assert!(!auto_grants_at_level(sc, 7).is_empty(), "level 7 Rally");
        }
    }

    #[test]
    fn capstones_are_subclass_exclusive() {
        let defense_12 = auto_grants_at_level("defense", 12);
        let two_handed_12 = auto_grants_at_level("two_handed", 12);
        let dueling_12 = auto_grants_at_level("dueling", 12);

        // Each subclass gets exactly one level-12 auto-grant
        assert_eq!(defense_12.len(), 1);
        assert_eq!(two_handed_12.len(), 1);
        assert_eq!(dueling_12.len(), 1); // Perfect Read ultimate

        // No overlap
        assert!(!two_handed_12.contains(&defense_12[0]));
        assert!(!dueling_12.contains(&defense_12[0]));
        assert!(!dueling_12.contains(&two_handed_12[0]));
    }

    #[test]
    fn op_test_card_follows_feature_flag() {
        let pool = reward_pool_at_level("dueling", 1);
        assert_eq!(pool.iter().any(|id| id == MARTIAL_ASCENDANCY), OP_TEST_CARD_ENABLED);
    }

    #[test]
    fn all_progression_card_ids_covers_table() {
        let all_ids = all_progression_card_ids();
        for entry in PROGRESSION_TABLE {
            assert!(
                all_ids.iter().any(|id| id == entry.card_id),
                "{} missing from all_progression_card_ids",
                entry.card_id
            );
        }
    }

    #[test]
    fn synergy_filter_marked_excludes_tempo() {
        let pool = reward_pool_at_level_filtered("dueling", 12, Some("marked"));
        // Tempo cards should be excluded.
        assert!(!pool.contains(&FEINT.into()), "feint is tempo");
        assert!(!pool.contains(&READ_AND_REACT.into()), "read_and_react is tempo");
        assert!(!pool.contains(&FLURRY_OF_CUTS.into()), "flurry_of_cuts is tempo");
        assert!(!pool.contains(&MOMENTUM.into()), "momentum is tempo");
        assert!(!pool.contains(&PERFECT_RHYTHM.into()), "perfect_rhythm is tempo");
        // Marked cards should be included.
        assert!(pool.contains(&MEASURED_STRIKE.into()), "measured_strike is marked");
        assert!(pool.contains(&RIPOSTE.into()), "riposte is marked");
        assert!(pool.contains(&EXPLOIT_OPENING.into()), "exploit_opening is marked");
        assert!(pool.contains(&COUP_DE_GRACE.into()), "coup_de_grace is marked");
        // Neutral/shared cards should be included.
        assert!(pool.contains(&QUICK_SLASH.into()), "quick_slash is shared");
    }

    #[test]
    fn synergy_filter_tempo_excludes_marked() {
        let pool = reward_pool_at_level_filtered("dueling", 12, Some("tempo"));
        assert!(!pool.contains(&MEASURED_STRIKE.into()), "measured_strike is marked");
        assert!(!pool.contains(&EXPLOIT_OPENING.into()), "exploit_opening is marked");
        assert!(!pool.contains(&COUP_DE_GRACE.into()), "coup_de_grace is marked");
        // Tempo cards should be included.
        assert!(pool.contains(&FEINT.into()), "feint is tempo");
        assert!(pool.contains(&MOMENTUM.into()), "momentum is tempo");
        assert!(pool.contains(&PERFECT_RHYTHM.into()), "perfect_rhythm is tempo");
        // Neutral/shared still included.
        assert!(pool.contains(&QUICK_SLASH.into()), "quick_slash is shared");
    }

    #[test]
    fn synergy_filter_none_matches_unfiltered() {
        for level in [1, 5, 12] {
            let unfiltered = reward_pool_at_level("dueling", level);
            let filtered = reward_pool_at_level_filtered("dueling", level, None);
            assert_eq!(unfiltered, filtered, "None filter should match unfiltered at level {}", level);
        }
    }

    #[test]
    fn active_card_ids_includes_starter_deck() {
        let ids = active_card_ids_for_run("dueling", None, "human", "soldier");
        assert!(ids.contains(&STRIKE.into()));
        assert!(ids.contains(&DEFEND.into()));
        assert!(ids.contains(&SECOND_WIND.into()));
        assert!(ids.contains(&IMPROVISE.into())); // human race card
    }

    #[test]
    fn active_card_ids_filtered_excludes_other_synergy() {
        let marked_ids = active_card_ids_for_run("dueling", Some("marked"), "human", "soldier");
        assert!(marked_ids.contains(&MEASURED_STRIKE.into()), "marked card present");
        assert!(marked_ids.contains(&STRIKE.into()), "basic present");
        assert!(marked_ids.contains(&DEFT_STRIKE.into()), "neutral auto-grant present");
        assert!(!marked_ids.contains(&FEINT.into()), "tempo card excluded");
        assert!(!marked_ids.contains(&PERFECT_RHYTHM.into()), "tempo capstone excluded");

        let tempo_ids = active_card_ids_for_run("dueling", Some("tempo"), "human", "soldier");
        assert!(tempo_ids.contains(&FEINT.into()), "tempo card present");
        assert!(!tempo_ids.contains(&MEASURED_STRIKE.into()), "marked card excluded");
    }

    #[test]
    fn active_card_ids_smaller_when_filtered() {
        let all = active_card_ids_for_run("dueling", None, "human", "soldier");
        let marked = active_card_ids_for_run("dueling", Some("marked"), "human", "soldier");
        let tempo = active_card_ids_for_run("dueling", Some("tempo"), "human", "soldier");
        assert!(marked.len() < all.len(), "marked filter should reduce card count");
        assert!(tempo.len() < all.len(), "tempo filter should reduce card count");
    }
}
