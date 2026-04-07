//! Combat screen layout — player on left, enemies on right, hand at bottom.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  "Combat - Turn X"                                          │
//! ├───────────────────┬─────────────────────────────────────────┤
//! │   PLAYER AREA     │          ENEMY AREA                     │
//! │   (280px fixed)   │          (flex-grow)                    │
//! │                   │                                         │
//! │   [Energy Orb]    │   [Enemy] [Enemy] [Enemy]               │
//! │   [HP bar]        │   intent / HP / status                  │
//! │   [Block]          │                                         │
//! │   [Status row]    │                                         │
//! ├───────────────────┴─────────────────────────────────────────┤
//! │  Targeting prompt                                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  [Card][Card][Card][Card][Card]  (hand row, 210px)          │
//! ├──────────────────────────────────────────────────┬──────────┤
//! │  [Draw Pile]  [Discard Pile]                     │ End Turn │
//! └──────────────────────────────────────────────────┴──────────┘
//! ```

use bevy::prelude::*;

use super::{CurrentCombatPalette, HandDirty, TargetingState};
use crate::plugins::run_start_ui::RunData;
use crate::theme;
use crate::tooltip::TooltipContent;
use crate::tooltip_text;

// ── Number of arrow segments for the Bézier targeting curve ──
pub const ARROW_SEGMENT_COUNT: usize = 20;

// ── Marker components ─────────────────────────────────────────

#[derive(Component)] pub struct CombatRoot;
#[derive(Component)] pub struct TurnText;
#[derive(Component)] pub struct BattlefieldArea;
#[derive(Component)] pub struct PlayerSpriteArea;
#[derive(Component)] pub struct PlayerSpriteImage;
#[derive(Component)] pub struct PlayerHpBar;
#[derive(Component)] pub struct PlayerHpText;
#[derive(Component)] pub struct PlayerEnergyText;
#[derive(Component)] pub struct PlayerBlockText;
#[derive(Component)] pub struct PlayerBlockContainer;
#[derive(Component)] pub struct EnergyOrb;
#[derive(Component)] pub struct PlayerStatusRow;

#[derive(Component)] pub struct EnemyArea;
#[derive(Component)] pub struct EnemyTarget(pub usize);
#[derive(Component)] pub struct EnemyHpText(pub usize);
#[derive(Component)] pub struct EnemyHpBar(pub usize);
#[derive(Component)] pub struct EnemyIntentContainer(pub usize);
#[derive(Component)] pub struct EnemyIntentText(pub usize);
#[derive(Component)] pub struct EnemyBlockText(pub usize);
#[derive(Component)] pub struct EnemyStatusRow(pub usize);
#[derive(Component)] pub struct EnemySpriteBox(pub usize);
#[derive(Component)] pub struct EnemySpriteImage(pub usize);

#[derive(Component)] pub struct HandRow;
#[derive(Component)] pub struct CardInHand(pub usize);

#[derive(Component)] pub struct DrawPileCard;
#[derive(Component)] pub struct DrawPileText;
#[derive(Component)] pub struct DiscardPileCard;
#[derive(Component)] pub struct DiscardPileText;
#[derive(Component)] pub struct ExhaustPileCard;
#[derive(Component)] pub struct ExhaustPileText;
#[derive(Component)] pub struct EndTurnButton;

#[derive(Component)] pub struct TargetingPrompt;
#[derive(Component)] pub struct TargetingArrowSegment(pub usize);
#[derive(Component)] pub struct LevelUpBannerNode;
#[derive(Component)] pub struct PerkBannerNode;
#[derive(Component)] pub struct PileViewerOverlay;
#[derive(Component)] pub struct ViewDeckButton;

// ── Setup system ──────────────────────────────────────────────

pub fn setup_combat(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    run_data: Option<Res<RunData>>,
    level_up: Option<Res<super::LevelUpBanner>>,
    perk_banner: Option<Res<super::PerkBanner>>,
) {
    commands.insert_resource(TargetingState::default());
    commands.insert_resource(HandDirty(true));
    commands.insert_resource(super::LastRenderedStatuses::default());

    // Select background + palette based on fight number
    let fight_num = run_data.as_ref().map(|rd| rd.runner.fights_won + 1).unwrap_or(1);
    let palette = theme::palette_for_fight(fight_num);
    commands.insert_resource(CurrentCombatPalette(palette));

    // If there's a level-up banner queued, spawn an overlay
    if let Some(lu) = level_up {
        let msg = format!("LEVEL UP!\n+5 Max HP (now {})\nHealed {} HP", lu.new_max_hp, lu.healed);
        commands.spawn((
            LevelUpBannerNode,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(100),
            BackgroundColor(theme::OVERLAY_BLACK),
        ))
        .with_children(|overlay| {
            // Inner card with dark background + gold border
            overlay.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(40.0), Val::Px(24.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme::BACKGROUND),
                BorderColor::all(theme::GOLD),
            ))
            .with_child((
                Text::new(msg),
                TextFont { font_size: 28.0, ..default() },
                TextColor(theme::GOLD),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
    }

    // If there's a perk banner queued, spawn an overlay (after level-up)
    if let Some(pb) = perk_banner {
        commands.spawn((
            PerkBannerNode,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(99),
            BackgroundColor(theme::OVERLAY_BLACK),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(40.0), Val::Px(24.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme::BACKGROUND),
                BorderColor::all(theme::TEAL),
            ))
            .with_child((
                Text::new(&pb.message),
                TextFont { font_size: 24.0, ..default() },
                TextColor(theme::TEAL),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
    }

    let obs = run_data.as_ref().map(|rd| rd.runner.observe());
    let (draw_count, discard_count, exhaust_count) = obs.as_ref()
        .map(|o| (o.draw_pile_size, o.discard_pile_size, o.exhaust_pile_card_ids.len()))
        .unwrap_or((0, 0, 0));

    commands
        .spawn((
            CombatRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // ── Top bar: fight number + turn counter ──────
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(44.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            })
            .with_children(|bar| {
                // Fight number + level (left)
                {
                    let level = obs.as_ref().map(|o| o.player_level).unwrap_or(1);
                    let draw_bonus = obs.as_ref().map(|o| o.bonus_draw).unwrap_or(0);
                    let energy_bonus = obs.as_ref().map(|o| o.bonus_energy).unwrap_or(0);
                    let mut info = format!("Fight {}/50  |  Lv {}", fight_num, level);
                    if draw_bonus > 0 {
                        info.push_str(&format!("  |  Draw +{}", draw_bonus));
                    }
                    if energy_bonus > 0 {
                        info.push_str(&format!("  |  Energy +{}", energy_bonus));
                    }
                    bar.spawn((
                        Text::new(info),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(theme::PARCHMENT),
                    ));
                }

                // Turn counter (center pill)
                bar.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(6.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.05, 0.10, 0.6)),
                ))
                .with_child((
                    TurnText,
                    Text::new(format!("Turn {}", obs.as_ref().map(|o| o.turn).unwrap_or(1))),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(theme::GOLD),
                ));

                // Deck button (right)
                bar.spawn((
                    ViewDeckButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.05, 0.10, 0.6)),
                    BorderColor::all(theme::BORDER_GOLD),
                ))
                .with_child((
                    Text::new("Deck"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(theme::PARCHMENT),
                ));
            });

            // ── Battlefield: player left, enemies right ────
            root.spawn((
                BattlefieldArea,
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    padding: UiRect::bottom(Val::Px(palette.ground_padding_bottom)),
                    ..default()
                },
                ImageNode::new(asset_server.load(palette.background_path)),
            ))
            .with_children(|battlefield| {
                // ── Player area (left, 280px) ────────
                spawn_player_area(battlefield, &obs, &asset_server, palette);

                // ── Enemy area (right, flex-grow) — positioning container ─────
                battlefield.spawn((
                    EnemyArea,
                    Node {
                        flex_grow: 1.0,
                        align_self: AlignSelf::Stretch, // fill parent height for absolute children
                        ..default()
                    },
                ))
                .with_children(|area| {
                    // Spawn enemy panels from initial observation
                    if let Some(ref o) = obs {
                        let alive_count = o.enemies.iter().filter(|e| e.alive).count();
                        // Look up per-slot offsets for this enemy count
                        let slot_offsets: Option<&[(f32, f32)]> = palette
                            .enemy_slot_offsets
                            .iter()
                            .find(|(count, _)| *count == alive_count)
                            .map(|(_, offs)| *offs);

                        let mut slot = 0usize;
                        for enemy in &o.enemies {
                            if !enemy.alive { continue; }
                            // Use explicit offsets if available, otherwise evenly space
                            let offset = slot_offsets
                                .and_then(|offs| offs.get(slot).copied())
                                .unwrap_or_else(|| {
                                    let spacing = 80.0 / (alive_count.max(1) as f32);
                                    (0.0, 5.0 + slot as f32 * spacing)
                                });
                            spawn_enemy_panel(area, enemy, &asset_server, palette, offset);
                            slot += 1;
                        }
                    }
                });
            });

            // ── Separator line between battlefield and hand ──
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(theme::BORDER_GOLD),
            ));

            // ── Targeting prompt (fixed height, always reserves space) ──
            root.spawn((
                TargetingPrompt,
                Text::new(""),
                TextFont { font_size: 18.0, ..default() },
                TextColor(theme::INTENT_ORANGE),
                Node {
                    align_self: AlignSelf::Center,
                    height: Val::Px(26.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));

            // ── Hand area: draw pile | hand cards | discard+exhaust + end turn ──
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(220.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|hand_area| {
                // Left: Draw pile
                hand_area.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|left| {
                    spawn_pile_card(
                        left, DrawPileCard, DrawPileText,
                        "Draw", draw_count, theme::TEAL,
                        tooltip_text::draw_pile_tooltip(draw_count),
                    );
                });

                // Center: Hand row (cards, flex-grow)
                hand_area.spawn((
                    HandRow,
                    Button,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::horizontal(Val::Px(8.0)),
                        ..default()
                    },
                ));
                // Cards are spawned dynamically by rebuild_hand system.

                // Right: Discard + Exhaust stacked, then End Turn
                hand_area.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|right| {
                    spawn_pile_card(
                        right, DiscardPileCard, DiscardPileText,
                        "Discard", discard_count, theme::SLATE,
                        tooltip_text::discard_pile_tooltip(discard_count),
                    );
                    spawn_pile_card(
                        right, ExhaustPileCard, ExhaustPileText,
                        "Exhaust", exhaust_count, theme::HEALTH_RED,
                        tooltip_text::exhaust_pile_tooltip(exhaust_count),
                    );

                    // End Turn button
                    right.spawn((
                        EndTurnButton,
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(theme::END_TURN_BG),
                        BorderColor::all(theme::GOLD),
                    ))
                    .with_child((
                        Text::new("End Turn"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(theme::GOLD),
                    ));
                });
            });

            // ── Targeting arrow segments (absolute, initially hidden) ──
            for i in 0..ARROW_SEGMENT_COUNT {
                root.spawn((
                    TargetingArrowSegment(i),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        left: Val::Px(-100.0),
                        top: Val::Px(-100.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(theme::ARROW_DEFAULT),
                    GlobalZIndex(50),
                    Visibility::Hidden,
                ));
            }
        });
}

// ── Pile card builder ─────────────────────────────────────────

fn spawn_pile_card(
    parent: &mut ChildSpawnerCommands,
    card_marker: impl Component,
    text_marker: impl Component,
    label: &str,
    count: usize,
    accent: Color,
    tooltip: String,
) {
    parent.spawn((
        card_marker,
        Button,
        TooltipContent(tooltip),
        Node {
            width: Val::Px(56.0),
            height: Val::Px(48.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        BorderColor::all(accent),
    ))
    .with_children(|card| {
        card.spawn((
            text_marker,
            Text::new(format!("{}", count)),
            TextFont { font_size: 18.0, ..default() },
            TextColor(theme::PARCHMENT),
        ));
        card.spawn((
            Text::new(label),
            TextFont { font_size: 10.0, ..default() },
            TextColor(accent),
        ));
    });
}

// ── Enemy panel builder ───────────────────────────────────────

fn spawn_enemy_panel(
    parent: &mut ChildSpawnerCommands,
    enemy: &decker_gauntlet::observation::EnemyObs,
    asset_server: &AssetServer,
    palette: &theme::CombatPalette,
    slot_offset: (f32, f32),
) {
    use decker_engine::enemy::IntentType;

    let (intent_icon, intent_str, intent_bg) = match &enemy.intent {
        Some(IntentType::Attack(dmg)) => ("/|\\", format!(" {}", dmg), theme::INTENT_ATTACK_BG),
        Some(IntentType::Defend(amt)) => ("[+]", format!(" {}", amt), theme::INTENT_DEFEND_BG),
        Some(IntentType::Buff(_st, n)) => ("^", format!(" +{}", n), theme::INTENT_BUFF_BG),
        Some(IntentType::Debuff(st, n)) => ("v", format!(" {} {}", st.abbreviation(), n), theme::INTENT_DEBUFF_BG),
        Some(IntentType::AttackDefend(atk, def)) => ("/|\\", format!(" {} [+] {}", atk, def), theme::INTENT_ATTACK_BG),
        Some(IntentType::BuffAllies(_st, n)) => ("^", format!(" Rally +{}", n), theme::INTENT_BUFF_BG),
        None => ("?", "??".to_string(), theme::INTENT_UNKNOWN_BG),
    };

    let intent_tooltip = tooltip_text::enemy_intent_tooltip(&enemy.intent);

    // Per-slot offset for visual depth staggering.
    // margin_bottom pushes UP from FlexEnd ground.
    // Absolute positioning: bottom = up from ground, left = % from left edge.
    let (offset_up, left_pct) = slot_offset;

    // Enemy column: stats on top, sprite on bottom (grounded)
    parent.spawn((
        EnemyTarget(enemy.index),
        Button,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(offset_up),
            left: Val::Percent(left_pct),
            width: Val::Px(200.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(2.0),
            ..default()
        },
        // No panel background — sprites stand directly on the battlefield
        BorderColor::all(Color::NONE),
    ))
    .with_children(|col| {
        // ── Stats overlay (top) ──────────────────────────
        col.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(2.0),
            padding: UiRect::all(Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        })
        .with_children(|stats| {
            // Intent badge
            stats.spawn((
                Button,
                EnemyIntentContainer(enemy.index),
                TooltipContent(intent_tooltip),
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(intent_bg),
                BorderColor::all(theme::INTENT_ORANGE),
            ))
            .with_children(|badge| {
                badge.spawn((
                    Text::new(intent_icon),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(theme::PARCHMENT),
                ));
                badge.spawn((
                    EnemyIntentText(enemy.index),
                    Text::new(&intent_str),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(theme::PARCHMENT),
                ));
            });

            // Enemy name
            stats.spawn((
                Text::new(&enemy.name),
                TextFont { font_size: 13.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            // HP bar
            let hp_pct = if enemy.max_hp > 0 {
                (enemy.hp as f32 / enemy.max_hp as f32 * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            };
            stats.spawn((
                Node {
                    width: Val::Px(140.0),
                    height: Val::Px(8.0),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(theme::BORDER_SLATE),
            ))
            .with_children(|bg| {
                bg.spawn((
                    EnemyHpBar(enemy.index),
                    Node {
                        width: Val::Percent(hp_pct),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme::HEALTH_RED),
                ));
            });

            // HP + Block row
            stats.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|row| {
                // HP text
                row.spawn((
                    EnemyHpText(enemy.index),
                    Text::new(format!("{}/{}", enemy.hp, enemy.max_hp)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(palette.enemy_hp_text),
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(palette.enemy_stat_bg),
                ));

                // Block text (updated in-place, initially empty)
                row.spawn((
                    EnemyBlockText(enemy.index),
                    Text::new(""),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(palette.enemy_defense_text),
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(palette.enemy_stat_bg),
                ));
            });

            // Status badges row
            stats.spawn((
                EnemyStatusRow(enemy.index),
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(3.0),
                    row_gap: Val::Px(2.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
        });

        // ── Sprite (bottom, grounded) ────────────────────
        let sprite_path = format!("sprites/enemies/{}.png", enemy.enemy_id);
        let has_sprite = std::path::Path::new("assets").join(&sprite_path).exists()
            || std::path::Path::new("../../assets").join(&sprite_path).exists();

        col.spawn((
            EnemySpriteBox(enemy.index),
            Node {
                width: Val::Px(150.0),
                height: Val::Px(150.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_children(|sprite_box| {
            if has_sprite {
                sprite_box.spawn((
                    EnemySpriteImage(enemy.index),
                    ImageNode::new(asset_server.load(&sprite_path)),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            } else {
                sprite_box.spawn((
                    Text::new(&enemy.name),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(theme::PARCHMENT),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            }
        });

        // ── Drop shadow ──
        col.spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(10.0),
                border_radius: BorderRadius::all(Val::Px(50.0)),
                margin: UiRect::top(Val::Px(-5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        ));
    });
}

// ── Player area builder ──────────────────────────────────────

fn spawn_player_area(
    parent: &mut ChildSpawnerCommands,
    obs: &Option<decker_gauntlet::observation::Observation>,
    asset_server: &AssetServer,
    palette: &theme::CombatPalette,
) {
    let (hp, max_hp, energy, max_energy, block) = obs
        .as_ref()
        .map(|o| (o.player_hp, o.player_max_hp, o.player_energy, o.player_max_energy, o.player_block))
        .unwrap_or((0, 0, 0, 0, 0));

    // Player column: stats on top, sprite on bottom (grounded)
    parent.spawn((
        PlayerSpriteArea,
        Button,
        Node {
            width: Val::Px(320.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::left(Val::Px(40.0)),
            ..default()
        },
    ))
    .with_children(|player| {
        // ── Stats overlay (top) ──────────────────────────
        player.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(2.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        })
        .with_children(|stats| {
            // Energy orb
            stats.spawn((
                EnergyOrb,
                Button,
                TooltipContent(tooltip_text::energy_tooltip(energy, max_energy)),
                Node {
                    width: Val::Px(50.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(25.0)),
                    ..default()
                },
                BackgroundColor(theme::ENERGY_ORB),
                BorderColor::all(theme::ENERGY_ORB_BORDER),
            ))
            .with_child((
                PlayerEnergyText,
                Text::new(format!("{}/{}", energy, max_energy)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            // HP bar
            let hp_frac = if max_hp > 0 { hp as f32 / max_hp as f32 } else { 0.0 };
            stats.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(12.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.10, 0.08, 0.14, 0.8)),
                BorderColor::all(theme::BORDER_SLATE),
            ))
            .with_children(|bg| {
                bg.spawn((
                    PlayerHpBar,
                    Node {
                        width: Val::Percent(hp_frac * 100.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(theme::HEALTH_RED),
                ));
            });

            // HP + Block row
            stats.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|row| {
                // HP text
                row.spawn((
                    PlayerHpText,
                    Text::new(format!("{}/{} HP", hp, max_hp)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(theme::HEALTH_RED),
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(palette.enemy_stat_bg),
                ));

                // Block
                let block_vis = if block > 0 { Visibility::Inherited } else { Visibility::Hidden };
                row.spawn((
                    PlayerBlockContainer,
                    Button,
                    TooltipContent(tooltip_text::block_tooltip(block)),
                    Node {
                        padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(palette.enemy_stat_bg),
                    block_vis,
                ))
                .with_child((
                    PlayerBlockText,
                    Text::new(format!("{}", block)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(palette.enemy_defense_text),
                ));
            });

            // Status badges row
            stats.spawn((
                PlayerStatusRow,
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(3.0),
                    row_gap: Val::Px(2.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
        });

        // ── Sprite (bottom, grounded) ────────────────────
        player.spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(200.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .with_child((
            PlayerSpriteImage,
            ImageNode::new(asset_server.load("sprites/player/fighter.png")),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));

        // ── Drop shadow ──
        player.spawn((
            Node {
                width: Val::Px(160.0),
                height: Val::Px(12.0),
                border_radius: BorderRadius::all(Val::Px(50.0)),
                margin: UiRect::top(Val::Px(-6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        ));
    });
}

// ── Cleanup ──────────────────────────────────────────────────

pub fn cleanup_combat(
    mut commands: Commands,
    query: Query<Entity, With<CombatRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<TargetingState>();
    commands.remove_resource::<HandDirty>();
    commands.remove_resource::<CurrentCombatPalette>();
    commands.remove_resource::<super::LastRenderedStatuses>();
    commands.remove_resource::<super::systems::PileViewerJustOpened>();
}
