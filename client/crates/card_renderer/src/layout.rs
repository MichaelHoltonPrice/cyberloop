//! Card layout builder — constructs the Bevy UI node tree for a card.
//!
//! Layout:
//! ```text
//! ┌──────────────────────────┐
//! │ ⬤                       │  ← energy cost orb (top-left)
//! │       { art icon }       │  ← art area (tag-tinted background)
//! ├── Card Name ─────────────┤  ← name banner
//! ├─────────────────────────────┤
//! │  Description text        │  ← description area
//! │              Exhaust  ◆  │
//! └──────────────────────────┘
//! ```

use bevy::prelude::*;

use decker_engine::card::{CardDef, Effect};

use crate::components::*;
use crate::theme;
use crate::CardDisplayConfig;

// --- Base design dimensions ---

const BASE_WIDTH: f32 = 200.0;
const BASE_HEIGHT: f32 = 280.0;
const BASE_BORDER: f32 = 3.0;
const BASE_RADIUS: f32 = 8.0;
const BASE_INNER_PAD: f32 = 3.0;

const ART_HEIGHT: f32 = 115.0;
const NAME_BANNER_HEIGHT: f32 = 26.0;

const COST_ORB_SIZE: f32 = 30.0;
const COST_ORB_INSET: f32 = 4.0;

const DESC_PADDING: f32 = 6.0;

const COST_FONT_SIZE: f32 = 16.0;
const NAME_FONT_SIZE: f32 = 16.0;
const DESC_FONT_SIZE: f32 = 13.0;
const EXHAUST_FONT_SIZE: f32 = 10.0;
const ART_ICON_FONT_SIZE: f32 = 32.0;

/// Spawns a complete card visual as a child of `parent`.
///
/// Returns the `Entity` of the root `CardWidget` node.
pub fn spawn_card(
    parent: &mut ChildSpawnerCommands,
    card_def: &CardDef,
    config: &CardDisplayConfig,
) -> Entity {
    let s = config.scale;

    let border_color = if config.dimmed {
        theme::BORDER_SLATE
    } else {
        theme::tag_border_color(&card_def.tags)
    };
    let bg_color = if config.dimmed {
        theme::CARD_DIMMED
    } else {
        theme::tag_background_tint(&card_def.tags)
    };
    let text_color = if config.dimmed {
        theme::CARD_DIMMED_TEXT
    } else {
        theme::PARCHMENT
    };
    let cost_color = if config.dimmed {
        theme::CARD_DIMMED_TEXT
    } else {
        theme::GOLD
    };

    let root_entity = parent
        .spawn((
            CardWidget,
            Node {
                width: Val::Px(BASE_WIDTH * s),
                height: Val::Px(BASE_HEIGHT * s),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(BASE_BORDER * s)),
                border_radius: BorderRadius::all(Val::Px(BASE_RADIUS * s)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(border_color),
        ))
        .with_children(|card| {
            spawn_art_area(card, card_def, s, cost_color, config);
            spawn_name_banner(card, card_def, s, text_color);
            spawn_description_panel(card, card_def, s, text_color, config);
        })
        .id();

    root_entity
}

fn spawn_art_area(
    card: &mut ChildSpawnerCommands,
    card_def: &CardDef,
    s: f32,
    cost_color: Color,
    config: &CardDisplayConfig,
) {
    let art_bg = if config.dimmed {
        theme::CARD_DIMMED
    } else {
        theme::tag_art_area_bg(&card_def.tags)
    };

    let accent = theme::class_accent_color(card_def.class);
    let accent_srgba = accent.to_srgba();

    card.spawn((
        CardArtArea,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(ART_HEIGHT * s),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(BASE_INNER_PAD * s)),
            overflow: Overflow::visible(),
            ..default()
        },
        BackgroundColor(art_bg),
    ))
    .with_children(|art| {
        if config.show_cost {
            spawn_cost_orbs(art, card_def, s, cost_color, config.dimmed);
        }

        let symbol = theme::art_placeholder_symbol(&card_def.tags, card_def.class);
        let icon_color = Color::srgba(
            accent_srgba.red,
            accent_srgba.green,
            accent_srgba.blue,
            0.25,
        );

        art.spawn(Node {
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_child((
            Text::new(symbol),
            TextFont { font_size: ART_ICON_FONT_SIZE * s, ..default() },
            TextColor(icon_color),
            TextLayout::new_with_justify(Justify::Center),
        ));
    });
}

fn spawn_cost_orbs(
    art: &mut ChildSpawnerCommands,
    card_def: &CardDef,
    s: f32,
    cost_color: Color,
    dimmed: bool,
) {
    let orb_size = COST_ORB_SIZE * s;
    let inset = COST_ORB_INSET * s;

    art.spawn((
        CardCostOrb,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(inset),
            top: Val::Px(inset),
            width: Val::Px(orb_size),
            height: Val::Px(orb_size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0 * s)),
            border_radius: BorderRadius::all(Val::Px(orb_size * 0.5)),
            ..default()
        },
        BackgroundColor(if dimmed { theme::CARD_DIMMED } else { theme::ENERGY_ORB }),
        BorderColor::all(if dimmed { theme::BORDER_SLATE } else { theme::ENERGY_ORB_BORDER }),
        ZIndex(1),
    ))
    .with_child((
        CardCostText,
        Text::new(format!("{}", card_def.cost)),
        TextFont { font_size: COST_FONT_SIZE * s, ..default() },
        TextColor(cost_color),
    ));

}

fn spawn_name_banner(
    card: &mut ChildSpawnerCommands,
    card_def: &CardDef,
    s: f32,
    text_color: Color,
) {
    let banner_bg = if text_color == theme::CARD_DIMMED_TEXT {
        theme::NAME_BANNER_BG
    } else {
        theme::tag_name_banner_bg(&card_def.tags)
    };

    card.spawn((
        CardTypeBanner,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(NAME_BANNER_HEIGHT * s),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(6.0 * s)),
            ..default()
        },
        BackgroundColor(banner_bg),
    ))
    .with_child((
        CardNameText,
        Text::new(&card_def.name),
        TextFont { font_size: NAME_FONT_SIZE * s, ..default() },
        TextColor(text_color),
        TextLayout::new_with_justify(Justify::Center),
    ));
}

fn spawn_description_panel(
    card: &mut ChildSpawnerCommands,
    card_def: &CardDef,
    s: f32,
    text_color: Color,
    config: &CardDisplayConfig,
) {
    card.spawn((
        Node {
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(DESC_PADDING * s)),
            ..default()
        },
        BackgroundColor(theme::DESC_AREA_BG),
    ))
    .with_children(|panel| {
        panel.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            ..default()
        })
        .with_child((
            CardDescriptionText,
            Text::new(&card_def.description),
            TextFont { font_size: DESC_FONT_SIZE * s, ..default() },
            TextColor(text_color),
        ));

        // Stats badges row (damage, block, draw, keywords)
        if !config.compact {
            let badges = compute_stat_badges(card_def);
            if !badges.is_empty() {
                panel.spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(3.0 * s),
                    row_gap: Val::Px(2.0 * s),
                    margin: UiRect::top(Val::Px(2.0 * s)),
                    ..default()
                })
                .with_children(|row| {
                    for (label, color) in &badges {
                        row.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(3.0 * s), Val::Px(1.0 * s)),
                                border_radius: BorderRadius::all(Val::Px(2.0 * s)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                        ))
                        .with_child((
                            Text::new(label.as_str()),
                            TextFont { font_size: 8.0 * s, ..default() },
                            TextColor(*color),
                        ));
                    }
                });
            }
        }

        if card_def.exhaust && !config.compact {
            panel.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(2.0 * s)),
                ..default()
            })
            .with_child((
                Text::new("Exhaust"),
                TextFont { font_size: EXHAUST_FONT_SIZE * s, ..default() },
                TextColor(theme::HEALTH_RED),
            ));
        }
    });
}

/// Compute stat badge labels from a card's effects.
fn compute_stat_badges(card_def: &CardDef) -> Vec<(String, Color)> {
    let mut badges = Vec::new();

    let total_damage: i32 = card_def.effects.iter().map(|e| match e {
        Effect::DealDamage { amount, .. } => *amount,
        Effect::DealDamageIfDamagedLastTurn { amount, .. } => *amount,
        Effect::DealDamageIfTargetDebuffed { amount, .. } => *amount,
        Effect::DealDamageIfEnemyBlocked { amount, .. } => *amount,
        Effect::DealDamageIfMarked { amount, .. } => *amount,
        Effect::DealDamagePerCardPlayed { per_card, .. } => *per_card,
        _ => 0,
    }).sum();

    let total_block: i32 = card_def.effects.iter().map(|e| match e {
        Effect::GainBlock { amount } => *amount,
        Effect::GainBlockIfDamagedLastTurn { amount } => *amount,
        Effect::BlockEqualMissingHp { max } => *max,
        _ => 0,
    }).sum();

    let draw_count: i32 = card_def.effects.iter().map(|e| match e {
        Effect::DrawCards { count } => *count as i32,
        Effect::DrawAndDiscard { draw, .. } => *draw as i32,
        _ => 0,
    }).sum();

    let energy_gain: i32 = card_def.effects.iter().map(|e| match e {
        Effect::GainEnergy { amount } => *amount,
        _ => 0,
    }).sum();

    if total_damage > 0 {
        badges.push((format!("{total_damage} DMG"), theme::HEALTH_RED));
    }
    if total_block > 0 {
        badges.push((format!("{total_block} BLK"), theme::DEFENSE_BLUE));
    }
    if draw_count > 0 {
        badges.push((format!("+{draw_count} Draw"), theme::TEAL));
    }
    if energy_gain > 0 {
        badges.push((format!("+{energy_gain} NRG"), theme::GOLD));
    }
    if card_def.innate {
        badges.push(("Innate".into(), theme::PARCHMENT));
    }
    if card_def.recycle {
        badges.push(("Recycle".into(), theme::TEAL));
    }

    badges
}

