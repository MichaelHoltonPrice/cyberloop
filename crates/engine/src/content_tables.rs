//! Immutable gameplay content loaded once at startup.
//!
//! Saved games and replays store only IDs and dynamic state;
//! definitions are resolved via this table at runtime.

use std::collections::HashMap;

use crate::card::CardDef;
use crate::card_ids::CardId;
use crate::enemy::EnemyDef;
use crate::ids::EnemyId;

#[derive(Debug, Clone)]
pub struct ContentTables {
    pub card_defs: HashMap<CardId, CardDef>,
    pub enemy_defs: HashMap<EnemyId, EnemyDef>,
}

impl ContentTables {
    /// Create empty content tables (useful for tests that supply their own defs).
    pub fn empty() -> Self {
        Self {
            card_defs: HashMap::new(),
            enemy_defs: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tables() {
        let tables = ContentTables::empty();
        assert!(tables.card_defs.is_empty());
        assert!(tables.enemy_defs.is_empty());
    }
}
