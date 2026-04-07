//! Decker content — card, enemy, encounter, and starter deck definitions.
//!
//! This crate provides the actual game data for the Decker engine.
//! It depends on `decker-engine` for type definitions (CardDef, EnemyDef, etc.)
//! and produces a `ContentTables` that the combat system consumes.

pub mod cards;
pub mod encounters;
pub mod enemies;
pub mod starter_decks;

use sha2::{Digest, Sha256};

use decker_engine::content_tables::ContentTables;
use decker_engine::subclass::fighter_subclasses;
use decker_engine::version::EngineVersion;

/// Load the default content set into a `ContentTables` ready for use by the
/// combat system. This is the primary entry point for consumers (gauntlet
/// runner, sim worker, etc.).
pub fn load_content() -> ContentTables {
    ContentTables {
        card_defs: cards::all_card_defs(),
        enemy_defs: enemies::all_enemy_defs(),
    }
}

/// Compute a SHA-256 digest over all gameplay-relevant content definitions.
///
/// Covers card defs, enemy defs, subclass defs, starter decks, reward pools,
/// and encounter pools — everything that determines how a given seed plays out.
/// The keys are sorted so the hash is deterministic across runs.
pub fn content_hash() -> String {
    let mut hasher = Sha256::new();

    // Card definitions (sorted by ID for determinism).
    let card_defs = cards::all_card_defs();
    let mut card_keys: Vec<_> = card_defs.keys().collect();
    card_keys.sort();
    for key in &card_keys {
        let json = serde_json::to_string(&card_defs[*key]).expect("card serialize");
        hasher.update(json.as_bytes());
    }

    // Enemy definitions (sorted by ID).
    let enemy_defs = enemies::all_enemy_defs();
    let mut enemy_keys: Vec<_> = enemy_defs.keys().collect();
    enemy_keys.sort();
    for key in &enemy_keys {
        let json = serde_json::to_string(&enemy_defs[*key]).expect("enemy serialize");
        hasher.update(json.as_bytes());
    }

    // Subclass definitions (sorted by ID).
    let mut subclasses = fighter_subclasses();
    subclasses.sort_by(|a, b| a.id.cmp(&b.id));
    let json = serde_json::to_string(&subclasses).expect("subclass serialize");
    hasher.update(json.as_bytes());

    // Starter decks (deterministic per subclass).
    for sc in &["defense", "dueling", "two_handed"] {
        let deck = starter_decks::fighter_starter_deck(sc, "human", "soldier");
        let json = serde_json::to_string(&deck).expect("starter deck serialize");
        hasher.update(json.as_bytes());
    }

    // Unified progression table (covers reward pool, auto-grants, and capstones).
    let progression_ids = cards::all_progression_card_ids();
    let json = serde_json::to_string(&progression_ids).expect("progression serialize");
    hasher.update(json.as_bytes());

    // Encounter pools (minion/standard/tough/boss IDs).
    let mut minions = enemies::minion_ids();
    minions.sort();
    hasher.update(serde_json::to_string(&minions).unwrap().as_bytes());

    let mut standards = enemies::standard_ids();
    standards.sort();
    hasher.update(serde_json::to_string(&standards).unwrap().as_bytes());

    let mut tough = enemies::tough_ids();
    tough.sort();
    hasher.update(serde_json::to_string(&tough).unwrap().as_bytes());

    for act in 1..=3u32 {
        let boss = enemies::boss_for_act(act);
        hasher.update(boss.as_bytes());
    }

    // Early encounters (fixed sequences).
    for i in 0..4usize {
        let enc = encounters::early_encounter(i);
        hasher.update(serde_json::to_string(&enc).unwrap().as_bytes());
    }

    // Curated encounter tables and metadata.
    for entry in encounters::encounter_table_hash_inputs() {
        hasher.update(entry.as_bytes());
    }

    let result = hasher.finalize();
    format!("{result:x}")
}

/// Build the current [`EngineVersion`] with the content hash computed from
/// all gameplay-relevant definitions in this crate.
pub fn engine_version() -> EngineVersion {
    EngineVersion::new(content_hash())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_content_non_empty() {
        let content = load_content();
        assert!(!content.card_defs.is_empty());
        assert!(!content.enemy_defs.is_empty());
    }

    #[test]
    fn all_starter_deck_cards_exist_in_content() {
        let content = load_content();
        for subclass in &["two_handed", "defense", "dueling"] {
            let deck = starter_decks::fighter_starter_deck(subclass, "human", "soldier");
            for card in &deck {
                assert!(
                    content.card_defs.contains_key(&card.def_id),
                    "starter card {:?} for {} not found in content",
                    card.def_id,
                    subclass
                );
            }
        }
    }

    #[test]
    fn all_encounter_enemies_exist_in_content() {
        let content = load_content();
        // Check early encounters
        for i in 0..4 {
            for id in encounters::early_encounter(i) {
                assert!(
                    content.enemy_defs.contains_key(&id),
                    "early encounter enemy {:?} not found in content",
                    id
                );
            }
        }
        // Check boss encounters
        for act in 1..=3 {
            for id in encounters::boss_encounter(act) {
                assert!(
                    content.enemy_defs.contains_key(&id),
                    "boss enemy {:?} not found in content",
                    id
                );
            }
        }
    }

    #[test]
    fn content_hash_is_deterministic() {
        let h1 = content_hash();
        let h2 = content_hash();
        assert_eq!(h1, h2, "content hash must be stable across calls");
        assert_eq!(h1.len(), 64, "SHA-256 hex digest should be 64 chars");
    }

    #[test]
    fn engine_version_is_well_formed() {
        let v = engine_version();
        assert!(!v.semver.is_empty());
        assert!(!v.git_commit.is_empty());
        assert_eq!(v.content_hash.len(), 64);
    }
}
