//! Feat system — passive permanent bonuses chosen at feat levels.
//! Currently a stub; feats were removed alongside ability scores.

use serde::{Deserialize, Serialize};

/// Feats available in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatId {}

impl FeatId {
    pub fn all() -> &'static [FeatId] {
        &[]
    }
}
