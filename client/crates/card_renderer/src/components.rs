//! Marker components for querying card sub-elements.

use bevy::prelude::*;

/// Root marker for any card widget spawned by the card renderer.
#[derive(Component)]
pub struct CardWidget;

/// Marks the cost orb node.
#[derive(Component)]
pub struct CardCostOrb;

/// Marks the cost text node inside the cost orb.
#[derive(Component)]
pub struct CardCostText;

/// Marks the card name text node.
#[derive(Component)]
pub struct CardNameText;

/// Marks the card art area.
#[derive(Component)]
pub struct CardArtArea;

/// Marks the name banner.
#[derive(Component)]
pub struct CardTypeBanner;

/// Marks the ability delivery strip.
#[derive(Component)]
pub struct CardAbilityStrip;

/// Marks the card description text node.
#[derive(Component)]
pub struct CardDescriptionText;
