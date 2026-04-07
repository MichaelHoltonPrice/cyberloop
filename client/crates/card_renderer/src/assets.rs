//! Card asset management.
//!
//! Holds handles to card frame images, icon sprites, and fonts.
//! All fields are `None` — the card renderer falls back to pure-Bevy UI
//! rendering. Future phases will populate these via asset loading.

use bevy::prelude::*;

/// Holds optional asset handles for card visuals.
///
/// When fields are `None`, the card renderer uses pure Bevy UI nodes
/// (rounded rectangles, colored backgrounds) as fallbacks.
#[derive(Resource, Default)]
pub struct CardAssets {
    pub frame_common: Option<Handle<Image>>,
    pub frame_uncommon: Option<Handle<Image>>,
    pub frame_rare: Option<Handle<Image>>,

    pub cost_orb: Option<Handle<Image>>,
    pub tag_attack: Option<Handle<Image>>,
    pub tag_defense: Option<Handle<Image>>,
    pub tag_skill: Option<Handle<Image>>,
    pub tag_power: Option<Handle<Image>>,

    pub name_font: Option<Handle<Font>>,
    pub body_font: Option<Handle<Font>>,
    pub cost_font: Option<Handle<Font>>,
}
