//! Class definitions — tabletop-fantasy archetypes.
//!
//! Only implemented classes appear in the enum. New classes are added
//! as variants when their mechanics are built, not before.

use serde::{Deserialize, Serialize};

/// The player's chosen class for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Class {
    Fighter,
    Barbarian,
    Monk,
    Warlock,
    Paladin,
    Rogue,
    Druid,
    Wizard,
    Sorcerer,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Fighter => "Fighter",
            Class::Barbarian => "Barbarian",
            Class::Monk => "Monk",
            Class::Warlock => "Warlock",
            Class::Paladin => "Paladin",
            Class::Rogue => "Rogue",
            Class::Druid => "Druid",
            Class::Wizard => "Wizard",
            Class::Sorcerer => "Sorcerer",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Class::Fighter => {
                "A master of martial combat, trained in a variety of weapons and armor."
            }
            Class::Barbarian => {
                "A berserker who channels pain into power. Getting hit makes you stronger."
            }
            Class::Monk => {
                "A martial artist who flows between stances, building Ki for devastating combos."
            }
            Class::Warlock => {
                "A pact-bound spellcaster who hexes foes and channels dark power through concentration."
            }
            Class::Paladin => {
                "A holy warrior who channels divine Smite charges to empower attacks with protective riders."
            }
            Class::Rogue => {
                "A cunning trickster who builds Artifice to empower devastating strikes."
            }
            Class::Druid => {
                "A nature shapeshifter who transforms into wild forms or channels concentration spells."
            }
            Class::Wizard => {
                "An arcane scholar who builds Arcane Charges with Skill spells, then unleashes them through devastating Attacks."
            }
            Class::Sorcerer => {
                "An innate spellcaster who builds Sorcery Points with every card, then detonates them in explosive payoffs."
            }
        }
    }

    pub fn subclass_prompt(self) -> &'static str {
        match self {
            Class::Fighter => "Choose Your Fighting Style",
            Class::Barbarian => "Choose Your Rage Path",
            Class::Monk => "Choose Your Monastic Tradition",
            Class::Warlock => "Choose Your Pact",
            Class::Paladin => "Choose Your Sacred Oath",
            Class::Rogue => "Choose Your Roguish Archetype",
            Class::Druid => "Choose Your Wild Shape",
            Class::Wizard => "Choose Your Arcane Tradition",
            Class::Sorcerer => "Choose Your Sorcerous Origin",
        }
    }

    /// Base starting HP for this class.
    pub fn base_hp(self) -> i32 {
        match self {
            Class::Fighter => 50,
            Class::Barbarian => 60,
            Class::Monk => 50,
            Class::Warlock => 45,
            Class::Paladin => 55,
            Class::Rogue => 45,
            Class::Druid => 55,
            Class::Wizard => 45,
            Class::Sorcerer => 50,
        }
    }

    /// HP gained per level-up.
    pub fn hp_per_level(self) -> i32 {
        match self {
            Class::Fighter => 8,
            Class::Barbarian => 9,
            Class::Monk => 7,
            Class::Warlock => 7,
            Class::Paladin => 8,
            Class::Rogue => 7,
            Class::Druid => 8,
            Class::Wizard => 6,
            Class::Sorcerer => 7,
        }
    }
}

/// Semantic tags on cards (used for class mechanic interactions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardTag {
    Attack,
    Defense,
    Skill,
    Power,
}
