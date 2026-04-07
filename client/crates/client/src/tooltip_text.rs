//! Centralized tooltip text for all UI elements.
//!
//! Each function returns a descriptive String for a specific game element.
//! Attach the result to a `TooltipContent` component on a `Button` entity.

use decker_engine::enemy::IntentType;
use decker_engine::status::StatusType;

/// Full tooltip for a status badge.
pub fn status_tooltip(status: &StatusType, stacks: i32) -> String {
    match status {
        StatusType::Threatened => format!(
            "Threatened ({stacks})\nTake +50% damage from all sources.\nDecays by 1 each turn."
        ),
        StatusType::Weakened => format!(
            "Weakened ({stacks})\nDeal -25% damage.\nDecays by 1 each turn."
        ),
        StatusType::Empowered => format!(
            "Empowered ({stacks})\n+{stacks} flat damage on attacks.\nPermanent."
        ),
        StatusType::Bleeding => format!(
            "Bleeding ({stacks})\nTake {stacks} damage at start of turn.\nBypasses block. Decays by 1."
        ),
        StatusType::Barbed => format!(
            "Barbed ({stacks})\nReflect {stacks} damage back to attackers.\nPermanent."
        ),
        StatusType::Mending => format!(
            "Mending ({stacks})\nHeal {stacks} HP at end of turn.\nDecays by 1 each turn."
        ),
        StatusType::Frightened => format!(
            "Frightened ({stacks})\nSkip next action.\nDecays by 1 each turn."
        ),
        StatusType::Marked => format!(
            "Marked ({stacks})\nTake +{stacks} damage from all sources.\nPermanent."
        ),
        StatusType::Armored => format!(
            "Armored ({stacks})\nReduces incoming damage by {stacks}.\nDecays by 1 each turn."
        ),
        StatusType::BlockRetention => format!(
            "Block Retention ({stacks})\nBlock is NOT reset at start of turn.\nAbsolute (does not decay)."
        ),
        StatusType::Momentum => format!(
            "Momentum ({stacks})\nDraw 1 card whenever you deal 12+\ndamage in a single hit.\nAbsolute (does not decay)."
        ),
        StatusType::SavageBlows => format!(
            "Savage Blows ({stacks})\n+{stacks} damage on single-target attacks.\nPermanent."
        ),
        StatusType::Rage => format!(
            "Rage ({stacks})\n+{stacks} flat damage on attacks.\nPermanent."
        ),
        StatusType::PainIsPower => format!(
            "Pain is Power ({stacks})\nWhen you lose HP, gain Rage equal\nto damage taken.\nAbsolute (does not decay)."
        ),
        StatusType::WardingTotem => format!(
            "Warding Totem ({stacks})\nWhen you lose HP, gain 3 block.\nAbsolute (does not decay)."
        ),
        StatusType::BerserkersTrance => format!(
            "Berserker's Trance ({stacks})\nStart of turn: lose 2 HP, draw 1 card.\nAbsolute (does not decay)."
        ),
        StatusType::Ki => format!(
            "Ki ({stacks})\nMonk resource. Spent by Ki abilities.\nPermanent."
        ),
        StatusType::StanceAggressive => format!(
            "Aggressive Stance ({stacks})\n+3 damage dealt, +3 damage taken.\nAbsolute (does not decay)."
        ),
        StatusType::StanceDefensive => format!(
            "Defensive Stance ({stacks})\n+3 block gained, -3 damage dealt.\nAbsolute (does not decay)."
        ),
        StatusType::StanceFlowing => format!(
            "Flowing Stance ({stacks})\nGain 1 Ki per card played.\nAbsolute (does not decay)."
        ),
        StatusType::Hexed => format!(
            "Hexed ({stacks})\nAttacks from the Warlock deal +3 damage.\nPermanent (does not decay)."
        ),
        StatusType::Smite => format!(
            "Smite ({stacks})\nSpend 1 charge on any attack for +8 damage\nplus a protective rider effect.\nEnduring."
        ),
        StatusType::Artifice => format!(
            "Artifice ({stacks})\nAt 10+, next single-target attack\ndeals bonus damage and consumes 10.\nEnduring."
        ),
        StatusType::WildShapeBear => format!(
            "Bear Form ({stacks})\nDefense cards gain +5 block.\nExclusive with other forms and concentration.\nAbsolute."
        ),
        StatusType::WildShapeEagle => format!(
            "Eagle Form ({stacks})\nFirst attack each turn gains +3 damage\nand draws 1 card.\nAbsolute."
        ),
        StatusType::WildShapeWolf => format!(
            "Wolf Form ({stacks})\nAttacks deal +3 damage to all targets.\nAbsolute."
        ),
        StatusType::ArcaneCharge => format!(
            "Arcane Charge ({stacks})\nBuilt by Skill cards, consumed by Attacks.\n1:+1, 2:+3, 3:+5, 4+:+5 and draw 1.\nEnduring."
        ),
        StatusType::Fortified => format!(
            "Fortified ({stacks})\nReduces incoming damage by {stacks}.\nEnduring."
        ),
        StatusType::SanctifiedShield => format!(
            "Sanctified Shield ({stacks})\nNegate the next incoming attack.\nAbsolute (consumed on use)."
        ),
    }
}

/// Tooltip for the energy orb.
pub fn energy_tooltip(energy: i32, max_energy: i32) -> String {
    format!(
        "Energy: {energy}/{max_energy}\nSpend energy to play cards.\nRefills to {max_energy} each turn.\nUnspent energy is lost."
    )
}

/// Tooltip for the block shield.
pub fn block_tooltip(block: i32) -> String {
    format!(
        "Block: {block}\nAbsorbs incoming damage.\nResets to 0 at start of your turn."
    )
}

/// Tooltip for the draw pile.
pub fn draw_pile_tooltip(count: usize) -> String {
    format!(
        "Draw Pile: {count} cards\nCards you will draw from.\nWhen empty, discard pile is shuffled in."
    )
}

/// Tooltip for the discard pile.
pub fn discard_pile_tooltip(count: usize) -> String {
    format!(
        "Discard Pile: {count} cards\nPlayed and discarded cards.\nShuffled into draw pile when it empties."
    )
}

/// Tooltip for the exhaust pile.
pub fn exhaust_pile_tooltip(count: usize) -> String {
    format!(
        "Exhaust Pile: {count} cards\nExhausted cards are removed for this combat.\nThey do not return to your deck."
    )
}

/// Tooltip for an enemy intent icon.
pub fn enemy_intent_tooltip(intent: &Option<IntentType>) -> String {
    match intent {
        Some(IntentType::Attack(dmg)) => {
            format!("Intent: Attack\nWill deal {dmg} damage.")
        }
        Some(IntentType::Defend(amt)) => {
            format!("Intent: Defend\nWill gain {amt} block.")
        }
        Some(IntentType::Buff(status, stacks)) => {
            format!("Intent: Buff\nWill apply {} ({stacks}).", status_name(status))
        }
        Some(IntentType::Debuff(status, stacks)) => {
            format!(
                "Intent: Debuff\nWill inflict {} ({stacks}) on you.",
                status_name(status)
            )
        }
        Some(IntentType::AttackDefend(atk, def)) => {
            format!(
                "Intent: Attack + Defend\nWill deal {atk} damage\nand gain {def} block."
            )
        }
        Some(IntentType::BuffAllies(status, stacks)) => {
            format!(
                "Intent: Rally\nWill grant {} ({stacks}) to all allies.",
                status_name(status)
            )
        }
        None => "Intent: Unknown\nThis enemy's next action is hidden.".into(),
    }
}

/// Brief one-line description for a status (used in card tooltips).
pub fn status_brief(status: &StatusType) -> &'static str {
    match status {
        StatusType::Threatened => "+50% damage taken, decays",
        StatusType::Weakened => "-25% damage dealt, decays",
        StatusType::Empowered => "+N flat attack damage, permanent",
        StatusType::Bleeding => "N damage/turn, bypasses block, decays",
        StatusType::Barbed => "reflect N damage to attackers, permanent",
        StatusType::Mending => "heal N HP/turn, decays",
        StatusType::Frightened => "skip next action, decays",
        StatusType::Marked => "+N damage from all sources, permanent",
        StatusType::Armored => "reduce incoming damage by N, decays",
        StatusType::BlockRetention => "block persists between turns",
        StatusType::Momentum => "draw 1 on 12+ damage hit",
        StatusType::SavageBlows => "+N single-target damage, permanent",
        StatusType::Rage => "+N flat attack damage, permanent",
        StatusType::PainIsPower => "gain Rage equal to HP lost",
        StatusType::WardingTotem => "gain 3 block when losing HP",
        StatusType::BerserkersTrance => "lose 2 HP, draw 1 card each turn",
        StatusType::Ki => "monk resource, spent by abilities",
        StatusType::StanceAggressive => "+3 damage dealt, +3 damage taken",
        StatusType::StanceDefensive => "+3 block gained, -3 damage dealt",
        StatusType::StanceFlowing => "gain 1 Ki per card played",
        StatusType::Hexed => "+3 damage from Warlock attacks, permanent",
        StatusType::Smite => "spend for +8 damage + protective effect",
        StatusType::Artifice => "at 10+, next attack deals bonus damage",
        StatusType::WildShapeBear => "+5 block on defense cards",
        StatusType::WildShapeEagle => "+3 damage + draw on first attack",
        StatusType::WildShapeWolf => "+3 damage to all targets",
        StatusType::ArcaneCharge => "build with Skills, consume on Attacks",
        StatusType::Fortified => "reduce incoming damage by N",
        StatusType::SanctifiedShield => "negate next incoming attack",
    }
}

/// Build a tooltip for a card, including mechanic explanations for any statuses.
pub fn card_tooltip(
    name: &str,
    description: &str,
    cost: i32,
    exhaust: bool,
    enemy_statuses: &[(StatusType, i32)],
    self_statuses: &[(StatusType, i32)],
) -> String {
    let mut lines = vec![
        format!("{name}  (Cost: {cost})"),
        description.to_string(),
    ];

    if exhaust {
        lines.push("Exhaust (removed for this combat after play)".into());
    }

    // Collect unique statuses for explanation
    let mut explained = std::collections::HashSet::new();
    for (st, _) in enemy_statuses.iter().chain(self_statuses.iter()) {
        if explained.insert(st) {
            lines.push(format!("  {} - {}", status_name(st), status_brief(st)));
        }
    }

    lines.join("\n")
}

/// Human-readable name for a StatusType.
pub fn status_name(status: &StatusType) -> &'static str {
    match status {
        StatusType::Threatened => "Threatened",
        StatusType::Weakened => "Weakened",
        StatusType::Empowered => "Empowered",
        StatusType::Bleeding => "Bleeding",
        StatusType::Barbed => "Barbed",
        StatusType::Mending => "Mending",
        StatusType::Frightened => "Frightened",
        StatusType::Marked => "Marked",
        StatusType::Armored => "Armored",
        StatusType::BlockRetention => "Block Retention",
        StatusType::Momentum => "Momentum",
        StatusType::SavageBlows => "Savage Blows",
        StatusType::Rage => "Rage",
        StatusType::PainIsPower => "Pain is Power",
        StatusType::WardingTotem => "Warding Totem",
        StatusType::BerserkersTrance => "Berserker's Trance",
        StatusType::Ki => "Ki",
        StatusType::StanceAggressive => "Aggressive Stance",
        StatusType::StanceDefensive => "Defensive Stance",
        StatusType::StanceFlowing => "Flowing Stance",
        StatusType::Hexed => "Hexed",
        StatusType::Smite => "Smite",
        StatusType::Artifice => "Artifice",
        StatusType::WildShapeBear => "Bear Form",
        StatusType::WildShapeEagle => "Eagle Form",
        StatusType::WildShapeWolf => "Wolf Form",
        StatusType::ArcaneCharge => "Arcane Charge",
        StatusType::Fortified => "Fortified",
        StatusType::SanctifiedShield => "Sanctified Shield",
    }
}
