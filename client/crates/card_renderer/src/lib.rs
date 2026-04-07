//! Card Graphic Creation Service for Decker.
//!
//! A standalone, reusable card rendering module that produces good-looking
//! card visuals at any scale using Bevy's UI system.
//!
//! # Usage
//!
//! ```rust,ignore
//! app.add_plugins(CardRendererPlugin);
//!
//! let config = CardDisplayConfig::combat_hand();
//! let entity = spawn_card(parent, &card_def, &config);
//! commands.entity(entity).insert((Button, CardInHand(idx)));
//! ```

pub mod assets;
pub mod components;
pub mod layout;
pub mod theme;

pub use assets::CardAssets;
pub use components::*;
pub use layout::spawn_card;

use bevy::prelude::*;

/// Registers the card renderer's resources. Add this to your Bevy app.
pub struct CardRendererPlugin;

impl Plugin for CardRendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardAssets>();
    }
}

/// Per-instance configuration for how a card should be displayed.
///
/// Cards are designed at a base size of 200x280 pixels. The `scale` factor
/// uniformly scales all dimensions, padding, borders, and font sizes.
#[derive(Clone, Debug)]
pub struct CardDisplayConfig {
    pub scale: f32,
    pub show_cost: bool,
    pub dimmed: bool,
    pub compact: bool,
}

impl Default for CardDisplayConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            show_cost: true,
            dimmed: false,
            compact: false,
        }
    }
}

impl CardDisplayConfig {
    /// Preset for combat hand cards (140x196 effective size).
    pub fn combat_hand() -> Self {
        Self { scale: 0.70, ..Default::default() }
    }

    /// Preset for reward screen cards (160x224 effective size).
    pub fn reward() -> Self {
        Self { scale: 0.80, ..Default::default() }
    }

    /// Preset for card removal grid (compact, 100x140 effective).
    pub fn removal() -> Self {
        Self { scale: 0.50, compact: true, ..Default::default() }
    }

    /// Preset for mini preview cards (100x140 effective, compact).
    pub fn mini() -> Self {
        Self { scale: 0.50, compact: true, ..Default::default() }
    }

    /// Returns a copy with the dimmed flag set.
    pub fn with_dimmed(mut self, dimmed: bool) -> Self {
        self.dimmed = dimmed;
        self
    }
}

/// Base card width before scaling (in design units).
pub const BASE_CARD_WIDTH: f32 = 200.0;

/// Base card height before scaling (in design units).
pub const BASE_CARD_HEIGHT: f32 = 280.0;

/// Returns the scaled card width for a given config.
pub fn card_width(config: &CardDisplayConfig) -> f32 {
    BASE_CARD_WIDTH * config.scale
}

/// Returns the scaled card height for a given config.
pub fn card_height(config: &CardDisplayConfig) -> f32 {
    BASE_CARD_HEIGHT * config.scale
}
