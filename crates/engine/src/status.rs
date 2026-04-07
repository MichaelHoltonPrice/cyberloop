use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Behavioral category for a status effect.
///
/// - **Waning**: decays 1 stack per turn, stacks accumulate.
/// - **Enduring**: permanent, stacks accumulate.
/// - **Absolute**: permanent, binary (clamped to 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusKind {
    Waning,
    Enduring,
    Absolute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusType {
    /// Take +50% damage. Waning.
    Threatened,
    /// Deal -25% damage. Waning.
    Weakened,
    /// +N flat damage on attacks. Enduring.
    Empowered,
    /// Damage-over-time: deal N damage (bypasses block), then decay. Waning.
    Bleeding,
    /// Reflect damage back to attacker on any hit. Enduring.
    Barbed,
    /// Heal-over-time (ticked separately via tick_mending). Waning.
    Mending,
    /// Enemy skips their action. Waning.
    Frightened,
    /// Take +1 damage per stack from all sources. Enduring.
    Marked,
    /// Temporary armor — each stack = +1 AC. Waning (decays each turn).
    Armored,
    /// Player block is NOT reset at start of turn. Absolute.
    BlockRetention,
    /// Draw 1 card whenever you deal 12+ damage in a single hit. Absolute.
    Momentum,
    /// +N damage on single-target attacks (like Empowered but single-target only). Enduring.
    SavageBlows,
    /// +N flat damage on attacks (Barbarian-specific, earned through pain). Enduring.
    Rage,
    /// When player loses HP, gain Rage equal to HP lost. Absolute.
    PainIsPower,
    /// When player loses HP, gain 3 block. Absolute.
    WardingTotem,
    /// Start of turn: lose 2 HP, draw 1 card. Absolute.
    BerserkersTrance,
    /// Monk Ki resource. Enduring — accumulated, spent by specific cards.
    Ki,
    /// Aggressive Stance: +3 attack damage dealt, +3 damage taken. Absolute.
    StanceAggressive,
    /// Defensive Stance: +3 block gained, -3 attack damage dealt. Absolute.
    StanceDefensive,
    /// Flowing Stance: gain 1 Ki per card played. Absolute.
    StanceFlowing,
    /// Hexed: takes bonus damage from attacks while caster holds Concentration. Absolute.
    Hexed,
    /// Paladin Smite resource. Enduring — starts at 2 per combat, spent by Smite cards.
    Smite,
    /// Rogue Artifice resource. Enduring — accumulated, spent by specific cards. Max 20.
    Artifice,
    /// Wild Shape Bear: +5 block on defense cards. Absolute.
    WildShapeBear,
    /// Wild Shape Eagle: draw 1 on first attack each turn. Absolute.
    WildShapeEagle,
    /// Wild Shape Wolf: +3 damage on attack cards. Absolute.
    WildShapeWolf,
    /// Wizard Arcane Charge resource. Enduring — accumulated by Skill cards, consumed by Attack cards.
    ArcaneCharge,
    /// Permanent damage reduction: each stack = -1 incoming damage. Enduring.
    Fortified,
    /// Negate the next enemy attack. Absolute — consumed on first hit.
    SanctifiedShield,
}

impl StatusType {
    /// The behavioral kind of this status.
    pub fn kind(&self) -> StatusKind {
        match self {
            Self::Threatened
            | Self::Weakened
            | Self::Bleeding
            | Self::Mending
            | Self::Frightened
            | Self::Armored => StatusKind::Waning,
            Self::Empowered | Self::Barbed | Self::SavageBlows | Self::Marked | Self::Rage | Self::Ki | Self::Smite | Self::Artifice
            | Self::ArcaneCharge | Self::Fortified => {
                StatusKind::Enduring
            }
            Self::BlockRetention | Self::Momentum | Self::PainIsPower | Self::WardingTotem | Self::BerserkersTrance
            | Self::StanceAggressive | Self::StanceDefensive | Self::StanceFlowing | Self::Hexed
            | Self::WildShapeBear | Self::WildShapeEagle | Self::WildShapeWolf
            | Self::SanctifiedShield => StatusKind::Absolute,
        }
    }

    /// Short abbreviation for UI badges.
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Self::Threatened => "THR",
            Self::Weakened => "WKN",
            Self::Empowered => "EMP",
            Self::Bleeding => "BLD",
            Self::Barbed => "BRB",
            Self::Mending => "MND",
            Self::Frightened => "FRI",
            Self::Marked => "MRK",
            Self::Armored => "ARM",
            Self::BlockRetention => "BRT",
            Self::Momentum => "MOM",
            Self::SavageBlows => "SVG",
            Self::Rage => "RGE",
            Self::PainIsPower => "PIP",
            Self::WardingTotem => "WTM",
            Self::BerserkersTrance => "BTR",
            Self::Ki => "KI",
            Self::StanceAggressive => "AGG",
            Self::StanceDefensive => "DEF",
            Self::StanceFlowing => "FLO",
            Self::Hexed => "HEX",
            Self::Smite => "SMT",
            Self::Artifice => "ART",
            Self::WildShapeBear => "WBR",
            Self::WildShapeEagle => "WEG",
            Self::WildShapeWolf => "WWF",
            Self::ArcaneCharge => "ARC",
            Self::Fortified => "FRT",
            Self::SanctifiedShield => "SSH",
        }
    }

    /// Whether this status is a buff (true) or debuff (false).
    pub fn is_buff(&self) -> bool {
        matches!(
            self,
            Self::Empowered
                | Self::Barbed
                | Self::Mending
                | Self::Armored
                | Self::BlockRetention
                | Self::Momentum
                | Self::SavageBlows
                | Self::Rage
                | Self::PainIsPower
                | Self::WardingTotem
                | Self::BerserkersTrance
                | Self::Ki
                | Self::StanceAggressive
                | Self::StanceDefensive
                | Self::StanceFlowing
                | Self::Smite
                | Self::Artifice
                | Self::WildShapeBear
                | Self::WildShapeEagle
                | Self::WildShapeWolf
                | Self::ArcaneCharge
                | Self::Fortified
                | Self::SanctifiedShield
        )
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusMap {
    statuses: HashMap<StatusType, i32>,
}

impl StatusMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&mut self, status: StatusType, stacks: i32) {
        match status.kind() {
            StatusKind::Absolute => {
                if stacks > 0 {
                    self.statuses.insert(status, 1);
                } else {
                    self.statuses.remove(&status);
                }
            }
            _ => {
                let entry = self.statuses.entry(status).or_insert(0);
                *entry += stacks;
                if *entry == 0 {
                    self.statuses.remove(&status);
                }
            }
        }
    }

    pub fn get(&self, status: StatusType) -> i32 {
        self.statuses.get(&status).copied().unwrap_or(0)
    }

    pub fn has(&self, status: StatusType) -> bool {
        self.statuses.contains_key(&status)
    }

    /// Tick statuses at end of turn.
    ///
    /// All **Waning** statuses decay by 1. Enduring and Absolute are untouched.
    /// Mending is ticked separately via `tick_mending()`.
    ///
    /// Returns `TickResult` with dot damage dealt this tick (Bleeding only).
    pub fn tick(&mut self) -> TickResult {
        let bleeding_damage = self.get(StatusType::Bleeding);

        let waning: Vec<StatusType> = self
            .statuses
            .keys()
            .filter(|s| s.kind() == StatusKind::Waning)
            .copied()
            .collect();
        for status in waning {
            self.apply(status, -1);
        }

        TickResult {
            dot_damage: bleeding_damage.max(0),
        }
    }

    /// Returns current Mending stacks (the amount to heal).
    /// Decay is handled by the generic `tick()` call.
    pub fn tick_mending(&self) -> i32 {
        self.get(StatusType::Mending).max(0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&StatusType, &i32)> {
        self.statuses.iter()
    }

    /// Returns true if any debuff status is active.
    pub fn has_any_debuff(&self) -> bool {
        self.statuses.keys().any(|s| !s.is_buff())
    }

    /// Count the number of distinct debuff types active with stacks > 0.
    pub fn debuff_count(&self) -> i32 {
        self.statuses.iter()
            .filter(|(&s, &v)| !s.is_buff() && v > 0)
            .count() as i32
    }

    pub fn clear(&mut self) {
        self.statuses.clear();
    }
}

/// Result of a status tick, containing damage values that need to be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickResult {
    pub dot_damage: i32,
}

impl TickResult {
    pub fn total_damage(&self) -> i32 {
        self.dot_damage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_and_get() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Empowered, 3);
        assert_eq!(map.get(StatusType::Empowered), 3);
        map.apply(StatusType::Empowered, 2);
        assert_eq!(map.get(StatusType::Empowered), 5);
    }

    #[test]
    fn negative_removal() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Threatened, 2);
        assert!(map.has(StatusType::Threatened));
        map.apply(StatusType::Threatened, -2);
        assert!(!map.has(StatusType::Threatened));
        assert_eq!(map.get(StatusType::Threatened), 0);
    }

    #[test]
    fn tick_decay() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Threatened, 2);
        map.apply(StatusType::Weakened, 1);
        map.apply(StatusType::Empowered, 3);

        let result = map.tick();
        assert_eq!(result.dot_damage, 0);
        assert_eq!(map.get(StatusType::Threatened), 1);
        assert!(!map.has(StatusType::Weakened));
        assert_eq!(map.get(StatusType::Empowered), 3);
    }

    #[test]
    fn tick_bleeding_damage() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Bleeding, 4);

        let result = map.tick();
        assert_eq!(result.dot_damage, 4);
        assert_eq!(map.get(StatusType::Bleeding), 3);
    }

    #[test]
    fn barbed_is_permanent() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Barbed, 3);
        let _ = map.tick();
        assert_eq!(map.get(StatusType::Barbed), 3);
    }

    #[test]
    fn frightened_decays() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Frightened, 2);
        let _ = map.tick();
        assert_eq!(map.get(StatusType::Frightened), 1);
        let _ = map.tick();
        assert!(!map.has(StatusType::Frightened));
    }

    #[test]
    fn marked_persists() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Marked, 3);
        let _ = map.tick();
        assert_eq!(map.get(StatusType::Marked), 3);
    }

    #[test]
    fn clear_removes_all() {
        let mut map = StatusMap::new();
        map.apply(StatusType::Empowered, 5);
        map.apply(StatusType::Bleeding, 2);
        map.apply(StatusType::Barbed, 1);
        map.apply(StatusType::Mending, 4);
        map.clear();
        assert_eq!(map.get(StatusType::Empowered), 0);
        assert_eq!(map.get(StatusType::Bleeding), 0);
        assert_eq!(map.get(StatusType::Barbed), 0);
        assert_eq!(map.get(StatusType::Mending), 0);
    }
}
