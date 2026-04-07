use bevy::prelude::*;

/// High-level game flow states for the Gauntlet MVP.
#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    RunStart,       // Character creation (multi-step wizard)
    Combat,         // In a gauntlet fight
    GauntletReward,     // Picking a reward card / removing / skipping
    DeckSwap,           // Swap a newly acquired card into the play deck
    CollectionOverflow, // Remove a card when collection exceeds max
    InnateChoice,       // Pick a card to always draw on turn 1
    DeckRebuild,        // Rebuild 16-card play deck from collection (rest stop)
    GameOver,           // Player died — show score
}
