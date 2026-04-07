//! Sub-classes — combat sub-specializations chosen at run start.
//!
//! Each sub-class defines starting cards and shapes the class's approach,
//! making it the primary axis of build identity after class selection.
//!
//! Cyberloop: stripped to Fighter sub-classes only.

use serde::{Deserialize, Serialize};

use crate::card_ids::CardId;
use crate::class::Class;
use crate::ids::SubclassId;

/// A sub-class tier level (0 = base, upgraded via elite/boss rewards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubclassTier {
    Tier0,
    Tier1,
    Tier2,
}

/// The player's active sub-class during a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSubclass {
    pub subclass_id: SubclassId,
    pub tier: SubclassTier,
}

/// Definition of a sub-class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassDef {
    pub id: SubclassId,
    pub name: String,
    pub description: String,
    /// Flavor text shown on the selection screen.
    pub flavor: String,
    pub class: Class,
    /// Cards granted by this sub-class.
    pub card_ids: Vec<CardId>,
}

// ── Fighter sub-class IDs ───────────────────────────────────────────────
pub const TWO_HANDED: &str = "two_handed";
pub const DEFENSE: &str = "defense";
pub const DUELING: &str = "dueling";

/// All Fighter sub-class definitions.
pub fn fighter_subclasses() -> Vec<SubclassDef> {
    vec![
        SubclassDef {
            id: TWO_HANDED.to_string(),
            name: "Two-Handed".to_string(),
            description: "Wield massive weapons for devastating strikes.".to_string(),
            flavor: "Strength over subtlety.".to_string(),
            class: Class::Fighter,
            card_ids: vec![],
        },
        SubclassDef {
            id: DEFENSE.to_string(),
            name: "Defense".to_string(),
            description: "Outlast your enemies behind heavy armor and shields.".to_string(),
            flavor: "An immovable wall.".to_string(),
            class: Class::Fighter,
            card_ids: vec![],
        },
        SubclassDef {
            id: DUELING.to_string(),
            name: "Dueling".to_string(),
            description: "Precise one-handed strikes with a free hand for defense.".to_string(),
            flavor: "Every opening exploited.".to_string(),
            class: Class::Fighter,
            card_ids: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fighter_subclass_ids_are_distinct() {
        let subs = fighter_subclasses();
        let ids: Vec<&str> = subs.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&TWO_HANDED));
        assert!(ids.contains(&DEFENSE));
        assert!(ids.contains(&DUELING));
    }
}
