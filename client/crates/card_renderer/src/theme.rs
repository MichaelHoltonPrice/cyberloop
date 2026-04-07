//! Card-specific theme constants and color helpers.

use bevy::prelude::*;

use decker_engine::card::{CardType, Rarity};
use decker_engine::class::{CardTag, Class};

// --- Base palette ---

pub const BACKGROUND: Color = Color::srgb(0.059, 0.055, 0.102);
pub const PANEL: Color = Color::srgb(0.102, 0.098, 0.157);
pub const PANEL_HOVER: Color = Color::srgb(0.141, 0.137, 0.212);
pub const PARCHMENT: Color = Color::srgb(0.910, 0.835, 0.718);
pub const GOLD: Color = Color::srgb(0.831, 0.647, 0.216);
pub const BORDER_GOLD: Color = Color::srgb(0.420, 0.353, 0.188);
pub const BORDER_SLATE: Color = Color::srgb(0.208, 0.208, 0.290);
pub const SLATE: Color = Color::srgb(0.290, 0.290, 0.369);
pub const TEAL: Color = Color::srgb(0.369, 0.612, 0.627);
pub const RARITY_UNCOMMON: Color = Color::srgb(0.369, 0.612, 0.627);
pub const RARITY_RARE: Color = Color::srgb(0.95, 0.75, 0.20);
pub const CARD_DIMMED: Color = Color::srgba(0.08, 0.07, 0.12, 0.90);
pub const CARD_DIMMED_TEXT: Color = Color::srgb(0.40, 0.38, 0.45);
pub const ENERGY_ORB: Color = Color::srgb(0.55, 0.15, 0.10);
pub const ENERGY_ORB_BORDER: Color = Color::srgb(0.75, 0.25, 0.15);
pub const DEFENSE_BLUE: Color = Color::srgb(0.302, 0.494, 0.780);
pub const HEALTH_RED: Color = Color::srgb(0.831, 0.220, 0.220);

// --- Card-specific colors ---

pub const NAME_BANNER_BG: Color = Color::srgba(0.04, 0.03, 0.08, 0.75);
pub const ART_AREA_BG: Color = Color::srgb(0.075, 0.070, 0.120);
pub const TYPE_BANNER_BG: Color = Color::srgba(0.08, 0.07, 0.12, 0.90);
pub const DESC_AREA_BG: Color = Color::srgba(0.16, 0.14, 0.11, 0.95);
pub const SEPARATOR: Color = Color::srgb(0.420, 0.353, 0.188);

// --- Tag-based card theming ---

pub fn tag_border_color(tags: &[CardTag]) -> Color {
    match tags.first() {
        Some(CardTag::Attack)  => Color::srgb(0.70, 0.25, 0.20),
        Some(CardTag::Defense) => Color::srgb(0.25, 0.45, 0.70),
        Some(CardTag::Skill)   => Color::srgb(0.35, 0.60, 0.30),
        Some(CardTag::Power)   => Color::srgb(0.55, 0.30, 0.65),
        None                   => BORDER_GOLD,
    }
}

pub fn tag_background_tint(tags: &[CardTag]) -> Color {
    match tags.first() {
        Some(CardTag::Attack)  => Color::srgb(0.12, 0.07, 0.07),
        Some(CardTag::Defense) => Color::srgb(0.07, 0.08, 0.13),
        Some(CardTag::Skill)   => Color::srgb(0.07, 0.10, 0.07),
        Some(CardTag::Power)   => Color::srgb(0.10, 0.07, 0.12),
        None                   => PANEL,
    }
}

pub fn tag_name_banner_bg(tags: &[CardTag]) -> Color {
    match tags.first() {
        Some(CardTag::Attack)  => Color::srgba(0.25, 0.08, 0.06, 0.85),
        Some(CardTag::Defense) => Color::srgba(0.06, 0.10, 0.25, 0.85),
        Some(CardTag::Skill)   => Color::srgba(0.08, 0.18, 0.08, 0.85),
        Some(CardTag::Power)   => Color::srgba(0.15, 0.08, 0.22, 0.85),
        None                   => NAME_BANNER_BG,
    }
}

pub fn tag_art_area_bg(tags: &[CardTag]) -> Color {
    match tags.first() {
        Some(CardTag::Attack)  => Color::srgb(0.10, 0.06, 0.06),
        Some(CardTag::Defense) => Color::srgb(0.06, 0.07, 0.11),
        Some(CardTag::Skill)   => Color::srgb(0.06, 0.09, 0.06),
        Some(CardTag::Power)   => Color::srgb(0.08, 0.06, 0.10),
        None                   => ART_AREA_BG,
    }
}

// --- Ki badge ---

pub const KI_BADGE_BG: Color = Color::srgba(0.35, 0.25, 0.08, 0.85);
pub const KI_BADGE_BORDER: Color = Color::srgb(0.60, 0.45, 0.15);

// --- Rarity helpers ---

pub fn rarity_border_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => BORDER_GOLD,
        Rarity::Uncommon => RARITY_UNCOMMON,
        Rarity::Rare => RARITY_RARE,
        Rarity::Legendary => Color::srgb(0.90, 0.55, 0.15),
    }
}

// --- Class helpers ---

pub fn class_accent_color(class: Option<Class>) -> Color {
    match class {
        Some(Class::Fighter) => Color::srgb(0.65, 0.40, 0.20),
        _ => SLATE,
    }
}

// --- Label helpers ---

pub fn card_type_label(card_type: CardType) -> &'static str {
    match card_type {
        CardType::Spell => "Spell",
        CardType::Consumable => "Consumable",
    }
}

pub fn tag_label(tag: CardTag) -> &'static str {
    match tag {
        CardTag::Attack => "Attack",
        CardTag::Defense => "Defense",
        CardTag::Skill => "Skill",
        CardTag::Power => "Power",
    }
}

/// Returns a text symbol as art placeholder based on card tags/class.
pub fn art_placeholder_symbol(tags: &[CardTag], class: Option<Class>) -> &'static str {
    if let Some(tag) = tags.first() {
        match tag {
            CardTag::Attack => match class {
                Some(Class::Fighter) => "/\\",
                _ => "/|\\",
            },
            CardTag::Defense => "[+]",
            CardTag::Skill => "<=>",
            CardTag::Power => "***",
        }
    } else {
        "..."
    }
}
