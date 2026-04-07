//! Named string identifiers for game content.
//!
//! All content IDs are human-readable strings (e.g. "goblin_grunt", "two_handed").
//! No arbitrary integers — IDs are self-describing in save files, logs, and tests.

/// Identifier for an enemy definition.
pub type EnemyId = String;

/// Identifier for a sub-class definition.
pub type SubclassId = String;
