//! Visual theme constants for Short Rest.
//!
//! Color palette: dark fantasy with wonder. Arcane manuscript aesthetic.

use bevy::prelude::*;

// --- Background & panels ---

/// Deep indigo/midnight — the darkness of the Pale Reach.
pub const BACKGROUND: Color = Color::srgb(0.059, 0.055, 0.102);

/// Dark panel — subtle separation from background.
pub const PANEL: Color = Color::srgb(0.102, 0.098, 0.157);

/// Panel on hover — slightly lighter for interaction feedback.
pub const PANEL_HOVER: Color = Color::srgb(0.141, 0.137, 0.212);

// --- Gold family ---

/// Warm gold — old spellbook gilding, arcane warmth. Primary accent.
pub const GOLD: Color = Color::srgb(0.831, 0.647, 0.216);

/// Soft amber glow — warm interaction feedback on hover.
pub const AMBER: Color = Color::srgb(0.769, 0.584, 0.227);

/// Dim gold — faint manuscript border feel.
pub const BORDER_GOLD: Color = Color::srgb(0.420, 0.353, 0.188);

// --- Text ---

/// Parchment cream — manuscript feel, readable against dark backgrounds.
pub const PARCHMENT: Color = Color::srgb(0.910, 0.835, 0.718);

// --- Accent ---

/// Ethereal teal — residual magic, wonder, the beauty in the ruins.
pub const TEAL: Color = Color::srgb(0.369, 0.612, 0.627);

// --- Disabled / muted ---

/// Muted slate — disabled, unclickable elements.
pub const SLATE: Color = Color::srgb(0.290, 0.290, 0.369);

/// Dim slate border — for disabled button borders.
pub const BORDER_SLATE: Color = Color::srgb(0.208, 0.208, 0.290);

// --- Combat ---

/// Health bar red.
pub const HEALTH_RED: Color = Color::srgb(0.831, 0.220, 0.220);

/// Defense / block blue.
pub const DEFENSE_BLUE: Color = Color::srgb(0.302, 0.494, 0.780);

/// Buff / positive status green.
pub const BUFF_GREEN: Color = Color::srgb(0.320, 0.702, 0.376);

/// Debuff / negative status purple.
#[allow(dead_code)]
pub const DEBUFF_PURPLE: Color = Color::srgb(0.608, 0.320, 0.780);

/// Enemy intent orange.
pub const INTENT_ORANGE: Color = Color::srgb(0.890, 0.557, 0.224);

/// Dark overlay backdrop for modals and dialogs.
pub const OVERLAY_BLACK: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);

// --- Card rarity borders ---

/// Uncommon rarity border — teal-ish.
#[allow(dead_code)]
pub const RARITY_UNCOMMON: Color = Color::srgb(0.369, 0.612, 0.627);

/// Rare rarity border — bright gold.
#[allow(dead_code)]
pub const RARITY_RARE: Color = Color::srgb(0.95, 0.75, 0.20);

// --- Card dimming (unplayable) ---

/// Dimmed card background — darker panel.
pub const CARD_DIMMED: Color = Color::srgba(0.08, 0.07, 0.12, 0.90);

/// Dimmed card text — muted.
#[allow(dead_code)]
pub const CARD_DIMMED_TEXT: Color = Color::srgb(0.40, 0.38, 0.45);

// --- Energy orb ---

/// Energy orb background — deep red-orange.
pub const ENERGY_ORB: Color = Color::srgb(0.55, 0.15, 0.10);

/// Energy orb border — brighter highlight.
pub const ENERGY_ORB_BORDER: Color = Color::srgb(0.75, 0.25, 0.15);

// --- End Turn button ---

/// End Turn button background — warm dark, stands out from panel.
pub const END_TURN_BG: Color = Color::srgb(0.14, 0.11, 0.08);

// --- Intent icons ---

/// Intent attack background — muted red.
pub const INTENT_ATTACK_BG: Color = Color::srgba(0.55, 0.15, 0.10, 0.6);

/// Intent defend background — muted blue.
pub const INTENT_DEFEND_BG: Color = Color::srgba(0.15, 0.30, 0.55, 0.6);

/// Intent buff background — muted green.
pub const INTENT_BUFF_BG: Color = Color::srgba(0.20, 0.45, 0.20, 0.6);

/// Intent debuff background — muted purple.
pub const INTENT_DEBUFF_BG: Color = Color::srgba(0.45, 0.20, 0.50, 0.6);

/// Intent unknown background — muted slate.
pub const INTENT_UNKNOWN_BG: Color = Color::srgba(0.20, 0.20, 0.28, 0.6);

// --- Targeting arrow ---

/// Arrow default color — gold.
pub const ARROW_DEFAULT: Color = Color::srgb(0.831, 0.647, 0.216);

/// Arrow valid target color — red (attack).
pub const ARROW_VALID: Color = Color::srgb(0.831, 0.220, 0.220);

// --- Status effect badges ---

/// Buff badge background — muted blue.
pub const STATUS_BUFF: Color = Color::srgb(0.22, 0.38, 0.58);

/// Debuff badge background — muted red.
pub const STATUS_DEBUFF: Color = Color::srgb(0.58, 0.22, 0.22);

// --- Tooltip ---

/// Tooltip background — near-black, slightly transparent.
pub const TOOLTIP_BG: Color = Color::srgba(0.06, 0.05, 0.10, 0.92);

/// Tooltip border — muted gold.
pub const TOOLTIP_BORDER: Color = Color::srgb(0.55, 0.43, 0.18);

// --- Combat background palettes ---

/// Colors that vary per combat background to keep enemy panels readable
/// and aesthetically matched to the scene.
#[derive(Clone, Debug)]
pub struct CombatPalette {
    /// Asset path for the background image.
    pub background_path: &'static str,
    /// Semi-transparent background for the overall enemy panel container.
    pub enemy_panel_bg: Color,
    /// Background behind stat text (HP, Block, AC).
    pub enemy_stat_bg: Color,
    /// Text color for HP values on enemy panels.
    pub enemy_hp_text: Color,
    /// Text color for Block/AC values on enemy panels.
    pub enemy_defense_text: Color,
    /// Border color for the enemy panel (idle, non-targeting state).
    pub enemy_panel_border: Color,
    /// Background for buff status badges.
    pub status_buff_bg: Color,
    /// Background for debuff status badges.
    pub status_debuff_bg: Color,
    /// Text color for status badges.
    pub status_text: Color,
    /// Padding from the bottom of the battlefield area to the visual ground
    /// line of the background image. Sprites align to this ground line.
    pub ground_padding_bottom: f32,
    /// Per-enemy-count slot offsets for staggered positioning (absolute).
    /// Each entry is `(enemy_count, per_slot_offsets)`.
    /// Per-slot offset is `(bottom_px, left_pct)`:
    /// `bottom_px` = distance from bottom in px (positive = up, negative = below ground);
    /// `left_pct` = percentage from left edge of enemy area (0 = left edge, 100 = right).
    pub enemy_slot_offsets: &'static [(usize, &'static [(f32, f32)])],
}

/// Combat palettes per act. Act 1 = Ruins, Act 2 = Lava, Act 3 = Lava (darker).
pub static COMBAT_PALETTES: &[CombatPalette] = &[
    // Act 1: Ruins — cool stone tones, early gauntlet
    CombatPalette {
        background_path: "backgrounds/combat_ruins.jpg",
        enemy_panel_bg: Color::srgba(0.03, 0.03, 0.08, 0.70),
        enemy_stat_bg: Color::srgba(0.02, 0.02, 0.06, 0.65),
        enemy_hp_text: PARCHMENT,
        enemy_defense_text: Color::srgb(0.50, 0.70, 0.95),
        enemy_panel_border: Color::srgba(0.35, 0.33, 0.45, 0.5),
        status_buff_bg: Color::srgba(0.15, 0.28, 0.52, 0.75),
        status_debuff_bg: Color::srgba(0.45, 0.15, 0.20, 0.75),
        status_text: Color::srgb(0.70, 0.85, 1.00),
        ground_padding_bottom: 20.0,
        enemy_slot_offsets: &[],
    },
    // Act 2: Lava — warm fiery tones, mid gauntlet
    CombatPalette {
        background_path: "backgrounds/combat_lava.jpg",
        enemy_panel_bg: Color::srgba(0.05, 0.02, 0.02, 0.75),
        enemy_stat_bg: Color::srgba(0.04, 0.02, 0.01, 0.70),
        enemy_hp_text: PARCHMENT,
        enemy_defense_text: Color::srgb(0.90, 0.75, 0.40),
        enemy_panel_border: Color::srgba(0.60, 0.30, 0.10, 0.5),
        status_buff_bg: Color::srgba(0.45, 0.32, 0.10, 0.75),
        status_debuff_bg: Color::srgba(0.50, 0.15, 0.10, 0.75),
        status_text: Color::srgb(1.00, 0.90, 0.70),
        ground_padding_bottom: 30.0,
        enemy_slot_offsets: &[
            (2, &[
                (-10.0, 15.0),
                (35.0, 50.0),
            ]),
            (3, &[
                (20.0, 5.0),
                (0.0, 23.0),
                (45.0, 45.0),
            ]),
            (4, &[
                (20.0, 5.0),
                (0.0, 23.0),
                (45.0, 45.0),
                (55.0, 65.0),
            ]),
            (5, &[
                (10.0, 0.0),
                (0.0, 18.0),
                (20.0, 36.0),
                (5.0, 54.0),
                (15.0, 72.0),
            ]),
        ],
    },
    // Act 3: Deep Lava — darker, more intense, late gauntlet
    CombatPalette {
        background_path: "backgrounds/combat_lava.jpg",
        enemy_panel_bg: Color::srgba(0.04, 0.01, 0.01, 0.80),
        enemy_stat_bg: Color::srgba(0.03, 0.01, 0.01, 0.75),
        enemy_hp_text: PARCHMENT,
        enemy_defense_text: Color::srgb(1.00, 0.60, 0.30),
        enemy_panel_border: Color::srgba(0.70, 0.20, 0.08, 0.6),
        status_buff_bg: Color::srgba(0.50, 0.25, 0.08, 0.80),
        status_debuff_bg: Color::srgba(0.60, 0.10, 0.08, 0.80),
        status_text: Color::srgb(1.00, 0.80, 0.55),
        ground_padding_bottom: 30.0,
        enemy_slot_offsets: &[
            (2, &[
                (-10.0, 15.0),
                (35.0, 50.0),
            ]),
            (3, &[
                (20.0, 5.0),
                (0.0, 23.0),
                (45.0, 45.0),
            ]),
            (4, &[
                (20.0, 5.0),
                (0.0, 23.0),
                (45.0, 45.0),
                (55.0, 65.0),
            ]),
            (5, &[
                (10.0, 0.0),
                (0.0, 18.0),
                (20.0, 36.0),
                (5.0, 54.0),
                (15.0, 72.0),
            ]),
        ],
    },
];

/// Select the palette for a given fight number (1-indexed).
/// Uses act-based selection: Act 1 (fights 1-6), Act 2 (fights 7-12), Act 3 (13+).
pub fn palette_for_fight(fight_num: u32) -> &'static CombatPalette {
    let fights_won = fight_num.saturating_sub(1);
    let act = 1 + (fights_won / 6).min(2);
    let index = (act as usize - 1).min(COMBAT_PALETTES.len() - 1);
    &COMBAT_PALETTES[index]
}
