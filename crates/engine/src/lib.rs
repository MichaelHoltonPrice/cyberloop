//! Decker Engine — pure game logic, no rendering dependencies.
//!
//! This crate implements the core state machine for the Decker deckbuilder:
//! combat resolution, card effects, enemy AI, and run progression.
//! It is designed for headless simulation, RL training, and game balance analysis.

pub mod card;
pub mod card_ids;
pub mod class;
pub mod combat;
pub mod content_tables;
pub mod enemy;
pub mod feat;
pub mod ids;
pub mod player;
pub mod rng;
pub mod status;
pub mod subclass;
pub mod version;
