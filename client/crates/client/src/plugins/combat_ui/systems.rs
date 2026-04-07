//! Combat systems — card play, targeting, display sync, hover effects, arrow.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use decker_engine::combat::Action;
use decker_gauntlet::{GauntletAction, GauntletEvent};

use super::setup::*;
use super::{HandDirty, TargetingState};
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;
use crate::tooltip::TooltipContent;
use crate::tooltip_text;

// ---------------------------------------------------------------------------
// Display sync — update player stats text from observation
// ---------------------------------------------------------------------------

// Type aliases for Without<T> filters to keep query signatures readable.
type NotTurn = Without<TurnText>;
type NotHp = Without<PlayerHpText>;
type NotEnergy = Without<PlayerEnergyText>;
type NotBlock = Without<PlayerBlockText>;
type NotDraw = Without<DrawPileText>;
type NotDiscard = Without<DiscardPileText>;
type NotExhaust = Without<ExhaustPileText>;

pub fn sync_combat_display(
    run_data: Option<Res<RunData>>,
    mut turn_q: Query<&mut Text, (With<TurnText>, NotHp, NotEnergy, NotDraw, NotDiscard, NotExhaust)>,
    mut hp_q: Query<&mut Text, (With<PlayerHpText>, NotTurn, NotEnergy, NotDraw, NotDiscard, NotExhaust)>,
    mut hp_bar_q: Query<&mut Node, With<PlayerHpBar>>,
    mut energy_q: Query<&mut Text, (With<PlayerEnergyText>, NotTurn, NotHp, NotDraw, NotDiscard, NotExhaust)>,
    mut block_q: Query<(&mut Text, &mut Visibility), (With<PlayerBlockText>, NotTurn, NotHp, NotEnergy, NotDraw, NotDiscard, NotExhaust)>,
    mut block_ctr_q: Query<&mut Visibility, (With<PlayerBlockContainer>, NotBlock)>,
    mut draw_q: Query<&mut Text, (With<DrawPileText>, NotTurn, NotHp, NotEnergy, NotDiscard, NotExhaust)>,
    mut discard_q: Query<&mut Text, (With<DiscardPileText>, NotTurn, NotHp, NotEnergy, NotDraw, NotExhaust)>,
    mut exhaust_q: Query<&mut Text, (With<ExhaustPileText>, NotTurn, NotHp, NotEnergy, NotDraw, NotDiscard)>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    if let Ok(mut text) = turn_q.single_mut() {
        **text = format!("Turn {}", obs.turn);
    }
    if let Ok(mut text) = hp_q.single_mut() {
        **text = format!("{}/{} HP", obs.player_hp, obs.player_max_hp);
    }
    if let Ok(mut node) = hp_bar_q.single_mut() {
        let frac = if obs.player_max_hp > 0 { obs.player_hp as f32 / obs.player_max_hp as f32 } else { 0.0 };
        node.width = Val::Percent(frac * 100.0);
    }
    if let Ok(mut text) = energy_q.single_mut() {
        **text = format!("{}/{}", obs.player_energy, obs.player_max_energy);
    }
    if let Ok((mut text, _vis)) = block_q.single_mut() {
        **text = format!("{}", obs.player_block);
    }
    if let Ok(mut vis) = block_ctr_q.single_mut() {
        *vis = if obs.player_block > 0 { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Ok(mut text) = draw_q.single_mut() {
        **text = format!("{}", obs.draw_pile_size);
    }
    if let Ok(mut text) = discard_q.single_mut() {
        **text = format!("{}", obs.discard_pile_size);
    }
    if let Ok(mut text) = exhaust_q.single_mut() {
        **text = format!("{}", obs.exhaust_pile_card_ids.len());
    }
}

// ---------------------------------------------------------------------------
// Sync enemy panels — update text/HP/intent/block/border IN PLACE (no rebuild)
// ---------------------------------------------------------------------------

pub fn sync_enemy_panels(
    run_data: Option<Res<RunData>>,
    targeting: Res<TargetingState>,
    _palette: Option<Res<super::CurrentCombatPalette>>,
    mut hp_text_query: Query<(&EnemyHpText, &mut Text), Without<EnemyIntentText>>,
    mut hp_bar_query: Query<(&EnemyHpBar, &mut Node)>,
    mut intent_query: Query<(&EnemyIntentText, &mut Text)>,
    mut intent_bg_query: Query<(&EnemyIntentContainer, &mut BackgroundColor)>,
    mut block_text_query: Query<(&EnemyBlockText, &mut Text), (Without<EnemyHpText>, Without<EnemyIntentText>)>,
    mut panel_query: Query<(&EnemyTarget, &Interaction, &mut BorderColor), Without<EnemySpriteBox>>,
    mut sprite_query: Query<(&EnemySpriteBox, &mut BorderColor), Without<EnemyTarget>>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();
    let is_targeting = targeting.selected_card_index.is_some();

    // Update HP text
    for (hp_text, mut text) in &mut hp_text_query {
        let idx = hp_text.0;
        if let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) {
            if !enemy.alive {
                **text = "DEAD".into();
            } else {
                **text = format!("{}/{}", enemy.hp, enemy.max_hp);
            }
        }
    }

    // Update HP bar
    for (hp_bar, mut node) in &mut hp_bar_query {
        let idx = hp_bar.0;
        if let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) {
            let frac = if enemy.max_hp > 0 && enemy.alive {
                (enemy.hp as f32 / enemy.max_hp as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            node.width = Val::Percent(frac * 100.0);
        }
    }

    // Update intent text
    for (intent_text, mut text) in &mut intent_query {
        let idx = intent_text.0;
        if let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) {
            if !enemy.alive {
                **text = "".into();
            } else {
                **text = format_intent(&enemy.intent);
            }
        }
    }

    // Update intent badge background color
    for (container, mut bg) in &mut intent_bg_query {
        let idx = container.0;
        if let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) {
            *bg = BackgroundColor(intent_bg_color(&enemy.intent, enemy.alive));
        }
    }

    // Update block text
    for (block_text, mut text) in &mut block_text_query {
        let idx = block_text.0;
        if let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) {
            if enemy.block > 0 && enemy.alive {
                **text = format!("Block: {}", enemy.block);
            } else {
                **text = "".into();
            }
        }
    }

    // Highlight enemy columns during targeting (no panel bg, so keep border transparent
    // by default and only show it when targeting)
    for (panel, interaction, mut border) in &mut panel_query {
        let idx = panel.0;
        let alive = obs.enemies.iter().find(|e| e.index == idx).is_some_and(|e| e.alive);
        if is_targeting && alive && *interaction == Interaction::Hovered {
            *border = BorderColor::all(theme::GOLD);
        } else if is_targeting && alive {
            *border = BorderColor::all(theme::INTENT_ORANGE);
        } else {
            *border = BorderColor::all(Color::NONE);
        }
    }

    // Also update sprite box borders during targeting
    for (sprite, mut border) in &mut sprite_query {
        let idx = sprite.0;
        let alive = obs.enemies.iter().find(|e| e.index == idx).is_some_and(|e| e.alive);
        if is_targeting && alive {
            *border = BorderColor::all(theme::INTENT_ORANGE);
        } else {
            *border = BorderColor::all(Color::NONE);
        }
    }
}

// ---------------------------------------------------------------------------
// Swap enemy sprite to death pose when HP reaches 0
// ---------------------------------------------------------------------------

/// Marker added to an enemy sprite after its dead image has been applied,
/// so we only swap once.
#[derive(Component)]
pub(crate) struct DeadSpriteApplied;

pub fn swap_dead_sprites(
    mut commands: Commands,
    run_data: Option<Res<RunData>>,
    asset_server: Res<AssetServer>,
    mut sprite_query: Query<(Entity, &EnemySpriteImage, &mut ImageNode), Without<DeadSpriteApplied>>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    for (entity, sprite, mut image_node) in &mut sprite_query {
        let idx = sprite.0;
        let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) else { continue };
        if enemy.alive { continue; }

        // Try to load a dead sprite for this enemy (.png first, then .jpg)
        let dead_path = {
            let png = format!("sprites/enemies/{}_dead.png", enemy.enemy_id);
            let exists = |p: &str| {
                std::path::Path::new("assets").join(p).exists()
                    || std::path::Path::new("../../assets").join(p).exists()
            };
            if exists(&png) {
                Some(png)
            } else {
                let jpg = format!("sprites/enemies/{}_dead.jpg", enemy.enemy_id);
                if exists(&jpg) { Some(jpg) } else { None }
            }
        };

        if let Some(path) = dead_path {
            image_node.image = asset_server.load(path);
        }
        // Mark as applied so we don't re-check every frame
        commands.entity(entity).insert(DeadSpriteApplied);
    }
}

/// Marker added to the player sprite after death image has been applied.
#[derive(Component)]
pub(crate) struct PlayerDeadSpriteApplied;

pub fn swap_dead_player_sprite(
    mut commands: Commands,
    run_data: Option<Res<RunData>>,
    asset_server: Res<AssetServer>,
    mut sprite_query: Query<(Entity, &mut ImageNode), (With<PlayerSpriteImage>, Without<PlayerDeadSpriteApplied>)>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();
    if obs.player_hp > 0 { return; }

    for (entity, mut image_node) in &mut sprite_query {
        image_node.image = asset_server.load("sprites/player/fighter_dead.png");
        commands.entity(entity).insert(PlayerDeadSpriteApplied);
    }
}

/// Background color for an intent badge based on intent type.
fn intent_bg_color(intent: &Option<decker_engine::enemy::IntentType>, alive: bool) -> Color {
    use decker_engine::enemy::IntentType;
    if !alive { return Color::NONE; }
    match intent {
        Some(IntentType::Attack(_)) | Some(IntentType::AttackDefend(_, _)) => theme::INTENT_ATTACK_BG,
        Some(IntentType::Defend(_)) => theme::INTENT_DEFEND_BG,
        Some(IntentType::Buff(_, _)) | Some(IntentType::BuffAllies(_, _)) => theme::INTENT_BUFF_BG,
        Some(IntentType::Debuff(_, _)) => theme::INTENT_DEBUFF_BG,
        None => theme::INTENT_UNKNOWN_BG,
    }
}

/// Format an intent for display (text portion only — icon is a separate node).
fn format_intent(intent: &Option<decker_engine::enemy::IntentType>) -> String {
    use decker_engine::enemy::IntentType;
    match intent {
        Some(IntentType::Attack(dmg)) => format!(" {}", dmg),
        Some(IntentType::Defend(amt)) => format!(" {}", amt),
        Some(IntentType::Buff(_st, n)) => format!(" +{}", n),
        Some(IntentType::Debuff(st, n)) => format!(" {} {}", st.abbreviation(), n),
        Some(IntentType::AttackDefend(atk, def)) => format!(" {} [+] {}", atk, def),
        Some(IntentType::BuffAllies(_st, n)) => format!(" Rally +{}", n),
        None => "??".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Sync player status badges
// ---------------------------------------------------------------------------

pub fn sync_player_statuses(
    mut commands: Commands,
    run_data: Option<Res<RunData>>,
    palette: Option<Res<super::CurrentCombatPalette>>,
    status_row_query: Query<Entity, With<PlayerStatusRow>>,
    mut last: ResMut<super::LastRenderedStatuses>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();
    let Ok(row_entity) = status_row_query.single() else { return };

    // Skip rebuild if nothing changed
    if obs.player_statuses == last.player
        && obs.block_floor == last.player_block_floor
        && obs.retain_block_cap == last.player_retain_cap
    {
        return;
    }
    last.player = obs.player_statuses.clone();
    last.player_block_floor = obs.block_floor;
    last.player_retain_cap = obs.retain_block_cap;

    let (buff_bg, debuff_bg, status_text) = palette
        .as_ref()
        .map(|p| (p.0.status_buff_bg, p.0.status_debuff_bg, p.0.status_text))
        .unwrap_or((theme::STATUS_BUFF, theme::STATUS_DEBUFF, theme::PARCHMENT));

    commands.entity(row_entity).despawn_children();
    commands.entity(row_entity).with_children(|row| {
        for (status_type, stacks) in &obs.player_statuses {
            if *stacks == 0 { continue; }
            let bg = if status_type.is_buff() { buff_bg } else { debuff_bg };
            row.spawn((
                Button,
                TooltipContent(tooltip_text::status_tooltip(status_type, *stacks)),
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(bg),
            ))
            .with_child((
                Text::new(format!("{} {}", status_type.abbreviation(), stacks)),
                TextFont { font_size: 11.0, ..default() },
                TextColor(status_text),
            ));
        }

        // Show block floor if active
        if obs.block_floor > 0 {
            row.spawn((
                Button,
                TooltipContent(format!(
                    "Unbreakable ({})\nBlock cannot drop below {}.",
                    obs.block_floor, obs.block_floor
                )),
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(buff_bg),
            ))
            .with_child((
                Text::new(format!("FLR {}", obs.block_floor)),
                TextFont { font_size: 11.0, ..default() },
                TextColor(status_text),
            ));
        }

        // Show retain block cap if active
        if obs.retain_block_cap > 0 {
            row.spawn((
                Button,
                TooltipContent(format!(
                    "Iron Fortress ({})\nRetain up to {} block between turns.",
                    obs.retain_block_cap, obs.retain_block_cap
                )),
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(buff_bg),
            ))
            .with_child((
                Text::new(format!("RTN {}", obs.retain_block_cap)),
                TextFont { font_size: 11.0, ..default() },
                TextColor(status_text),
            ));
        }
    });
}

// ---------------------------------------------------------------------------
// Sync enemy status badges (in-place rebuild of each enemy's status row)
// ---------------------------------------------------------------------------

pub fn sync_enemy_statuses(
    mut commands: Commands,
    run_data: Option<Res<RunData>>,
    palette: Option<Res<super::CurrentCombatPalette>>,
    status_row_query: Query<(Entity, &EnemyStatusRow)>,
    mut last: ResMut<super::LastRenderedStatuses>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    let (buff_bg, debuff_bg, status_text) = palette
        .as_ref()
        .map(|p| (p.0.status_buff_bg, p.0.status_debuff_bg, p.0.status_text))
        .unwrap_or((theme::STATUS_BUFF, theme::STATUS_DEBUFF, theme::PARCHMENT));

    for (row_entity, status_row) in &status_row_query {
        let idx = status_row.0;
        let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) else { continue };

        // Skip rebuild if this enemy's statuses haven't changed
        let current = if enemy.alive { enemy.statuses.clone() } else { Vec::new() };
        if last.enemies.get(&idx) == Some(&current) {
            continue;
        }
        last.enemies.insert(idx, current);

        commands.entity(row_entity).despawn_children();
        if !enemy.statuses.is_empty() && enemy.alive {
            commands.entity(row_entity).with_children(|row| {
                for (st, stacks) in &enemy.statuses {
                    if *stacks == 0 { continue; }
                    let bg = if st.is_buff() { buff_bg } else { debuff_bg };
                    row.spawn((
                        Button,
                        TooltipContent(tooltip_text::status_tooltip(st, *stacks)),
                        Node {
                            padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                    ))
                    .with_child((
                        Text::new(format!("{} {}", st.abbreviation(), stacks)),
                        TextFont { font_size: 10.0, ..default() },
                        TextColor(status_text),
                    ));
                }
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Rebuild hand — despawn old cards, spawn fresh from observation
// ---------------------------------------------------------------------------

pub fn rebuild_hand(
    mut commands: Commands,
    mut hand_dirty: ResMut<HandDirty>,
    run_data: Option<Res<RunData>>,
    hand_row_query: Query<Entity, With<HandRow>>,
) {
    if !hand_dirty.0 { return; }
    hand_dirty.0 = false;

    let Ok(hand_entity) = hand_row_query.single() else { return };
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    commands.entity(hand_entity).despawn_children();
    commands.entity(hand_entity).with_children(|hand| {
        for (i, card_obs) in obs.hand.iter().enumerate() {
            if let Some(card_def) = run_data.runner.content.card_defs.get(&card_obs.card_id) {
                let config = decker_card_renderer::CardDisplayConfig::combat_hand()
                    .with_dimmed(!card_obs.playable);
                let entity = decker_card_renderer::spawn_card(hand, card_def, &config);
                let tooltip = tooltip_text::card_tooltip(
                    &card_obs.name,
                    &card_obs.description,
                    card_obs.cost,
                    card_obs.exhaust,
                    &card_obs.enemy_statuses,
                    &card_obs.self_statuses,
                );
                hand.commands().entity(entity).insert((
                    Button,
                    CardInHand(i),
                    TooltipContent(tooltip),
                ));
            } else {
                spawn_text_card(hand, i, card_obs);
            }
        }
    });
}

/// Fallback text card when CardDef not found.
fn spawn_text_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    card: &decker_gauntlet::observation::CardObs,
) {
    let bg = if card.playable { theme::PANEL } else { theme::CARD_DIMMED };
    let border = if card.playable { theme::GOLD } else { theme::BORDER_SLATE };
    let text_color = if card.playable { theme::PARCHMENT } else { theme::SLATE };

    parent.spawn((
        CardInHand(index),
        Button,
        Node {
            width: Val::Px(140.0),
            min_height: Val::Px(196.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(bg),
        BorderColor::all(border),
    ))
    .with_children(|card_node| {
        card_node.spawn((
            Text::new(format!("{}", card.cost)),
            TextFont { font_size: 16.0, ..default() },
            TextColor(theme::GOLD),
        ));
        card_node.spawn((
            Text::new(&card.name),
            TextFont { font_size: 14.0, ..default() },
            TextColor(text_color),
        ));
        card_node.spawn((
            Text::new(&card.description),
            TextFont { font_size: 10.0, ..default() },
            TextColor(text_color),
        ));
    });
}

// ---------------------------------------------------------------------------
// Card hover effects — scale up on hover, dim when unplayable
// ---------------------------------------------------------------------------

pub fn card_hover_effects(
    run_data: Option<Res<RunData>>,
    targeting: Res<TargetingState>,
    mut card_query: Query<
        (&Interaction, &CardInHand, &mut Node, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    let base = decker_card_renderer::CardDisplayConfig::combat_hand();
    let base_w = decker_card_renderer::card_width(&base);
    let base_h = decker_card_renderer::card_height(&base);
    let hover_scale = base.scale * 1.07;
    let hover_w = decker_card_renderer::BASE_CARD_WIDTH * hover_scale;
    let hover_h = decker_card_renderer::BASE_CARD_HEIGHT * hover_scale;

    for (interaction, card_in_hand, mut node, mut bg, mut border) in &mut card_query {
        let idx = card_in_hand.0;
        let playable = obs.hand.get(idx).map(|c| c.playable).unwrap_or(false);
        let is_selected = targeting.selected_card_index == Some(idx);

        if !playable {
            node.width = Val::Px(base_w);
            node.height = Val::Px(base_h);
            node.margin = UiRect::ZERO;
            *bg = BackgroundColor(theme::CARD_DIMMED);
            *border = BorderColor::all(theme::BORDER_SLATE);
        } else if is_selected {
            node.width = Val::Px(hover_w);
            node.height = Val::Px(hover_h);
            node.margin = UiRect::vertical(Val::Px(-5.0));
            *bg = BackgroundColor(theme::PANEL_HOVER);
            *border = BorderColor::all(theme::GOLD);
        } else if *interaction == Interaction::Hovered {
            node.width = Val::Px(hover_w);
            node.height = Val::Px(hover_h);
            node.margin = UiRect::vertical(Val::Px(-5.0));
            *bg = BackgroundColor(theme::PANEL_HOVER);
            *border = BorderColor::all(theme::AMBER);
        } else {
            node.width = Val::Px(base_w);
            node.height = Val::Px(base_h);
            node.margin = UiRect::ZERO;
            *bg = BackgroundColor(theme::PANEL);
            *border = BorderColor::all(theme::BORDER_GOLD);
        }
    }
}

// ---------------------------------------------------------------------------
// Targeting arrow — quadratic Bézier from selected card to cursor
// ---------------------------------------------------------------------------

pub fn update_targeting_arrow(
    targeting: Res<TargetingState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    camera_query: Query<&Camera, With<Camera2d>>,
    card_query: Query<(&CardInHand, &UiGlobalTransform, &ComputedNode)>,
    mut arrow_query: Query<(&TargetingArrowSegment, &mut Node, &mut Visibility, &mut BackgroundColor), Without<CardInHand>>,
    enemy_query: Query<(&EnemyTarget, &Interaction)>,
) {
    let is_targeting = targeting.selected_card_index.is_some();

    if !is_targeting {
        // Hide all segments
        for (_, _, mut vis, _) in &mut arrow_query {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let hand_index = targeting.selected_card_index.unwrap();

    let Ok(window) = windows.single() else { return };
    let scale = ui_scale.0 as f32;

    // Compute viewport offset in logical pixels
    let viewport_offset = camera_query
        .single()
        .ok()
        .and_then(|camera| camera.viewport.as_ref())
        .map(|vp| {
            let sf = window.scale_factor();
            Vec2::new(
                vp.physical_position.x as f32 / sf,
                vp.physical_position.y as f32 / sf,
            )
        })
        .unwrap_or(Vec2::ZERO);

    // Find the selected card's position using UiGlobalTransform.
    // UiGlobalTransform gives the node's center in physical pixels.
    // Convert to Val::Px coords: physical / (scale_factor * ui_scale).
    let sf = window.scale_factor();
    let combined_sf = sf * scale;
    let start_ui = card_query.iter()
        .find(|(card, _, _)| card.0 == hand_index)
        .map(|(_, ui_gt, _computed)| {
            let translation = ui_gt.translation;
            // translation is in physical pixels → convert to Val::Px space
            translation / combined_sf
        });

    let Some(start) = start_ui else {
        for (_, _, mut vis, _) in &mut arrow_query {
            *vis = Visibility::Hidden;
        }
        return;
    };

    // Get cursor position — convert to UI-reference coords
    // window.cursor_position() returns logical window pixels
    let Some(cursor_pos) = window.cursor_position() else {
        for (_, _, mut vis, _) in &mut arrow_query {
            *vis = Visibility::Hidden;
        }
        return;
    };
    let end = (cursor_pos - viewport_offset) / scale;

    // Check if hovering a valid enemy
    let hovering_valid = enemy_query.iter().any(|(_, interaction)| {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    });

    let arrow_color = if hovering_valid {
        theme::ARROW_VALID
    } else {
        theme::ARROW_DEFAULT
    };

    // Compute Bézier control point (arcs upward)
    let midpoint = (start + end) / 2.0;
    let control = Vec2::new(midpoint.x, midpoint.y - 150.0);

    for (seg, mut node, mut vis, mut bg) in &mut arrow_query {
        let i = seg.0;
        let t = i as f32 / (ARROW_SEGMENT_COUNT - 1).max(1) as f32;
        let inv = 1.0 - t;

        // Quadratic Bézier: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
        let pos = start * (inv * inv) + control * (2.0 * inv * t) + end * (t * t);

        // Size varies — thicker in the middle
        let size_factor = 1.0 + 1.5 * (1.0 - (2.0 * t - 1.0).abs());
        let px = 3.0 * size_factor;

        node.left = Val::Px(pos.x - px);
        node.top = Val::Px(pos.y - px);
        node.width = Val::Px(px * 2.0);
        node.height = Val::Px(px * 2.0);
        *vis = Visibility::Inherited;
        *bg = BackgroundColor(arrow_color);
    }
}

// ---------------------------------------------------------------------------
// Targeting visuals — prompt text + End Turn border color
// ---------------------------------------------------------------------------

pub fn update_targeting_visuals(
    targeting: Res<TargetingState>,
    run_data: Option<Res<RunData>>,
    mut prompt_query: Query<&mut Text, With<TargetingPrompt>>,
    mut end_turn_query: Query<&mut BorderColor, With<EndTurnButton>>,
) {
    let Ok(mut prompt) = prompt_query.single_mut() else { return };

    // Check for forced discard mode
    let pending = run_data.as_ref()
        .map(|rd| rd.runner.observe().pending_discards)
        .unwrap_or(0);

    if pending > 0 {
        **prompt = format!("Click a card to discard ({} remaining)", pending);
        if let Ok(mut border) = end_turn_query.single_mut() {
            *border = BorderColor::all(theme::BORDER_SLATE);
        }
        return;
    }

    if targeting.selected_card_index.is_some() {
        **prompt = "Click a target to play the selected card".to_string();
        if let Ok(mut border) = end_turn_query.single_mut() {
            *border = BorderColor::all(theme::BORDER_SLATE);
        }
    } else {
        **prompt = String::new();
        if let Ok(mut border) = end_turn_query.single_mut() {
            *border = BorderColor::all(theme::GOLD);
        }
    }
}

// ---------------------------------------------------------------------------
// Hand row click — deselect card when clicking empty hand area
// ---------------------------------------------------------------------------

pub fn handle_hand_deselect(
    mut targeting: ResMut<TargetingState>,
    hand_query: Query<&Interaction, (Changed<Interaction>, With<HandRow>, Without<CardInHand>)>,
) {
    for interaction in &hand_query {
        if *interaction == Interaction::Pressed {
            targeting.selected_card_index = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Card click — select/deselect, or discard in forced-discard mode
// ---------------------------------------------------------------------------

pub fn handle_card_click(
    mut commands: Commands,
    run_data: Option<ResMut<RunData>>,
    mut targeting: ResMut<TargetingState>,
    mut hand_dirty: ResMut<HandDirty>,
    card_query: Query<(&Interaction, &CardInHand), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(mut run_data) = run_data else { return };

    for (interaction, card) in &card_query {
        if *interaction != Interaction::Pressed { continue; }

        let obs = run_data.runner.observe();
        let hand_index = card.0;
        if hand_index >= obs.hand.len() { continue; }

        // Forced discard mode: clicking a card discards it
        if obs.pending_discards > 0 {
            let action = GauntletAction::CombatAction(Action::DiscardCard { hand_index });
            if let Ok(events) = run_data.runner.apply(action) {
                hand_dirty.0 = true;
                check_combat_end(&events, &mut run_data, &mut next_state, &mut commands);
            }
            return;
        }

        if !obs.hand[hand_index].playable { continue; }

        // Toggle selection
        if targeting.selected_card_index == Some(hand_index) {
            targeting.selected_card_index = None;
        } else {
            targeting.selected_card_index = Some(hand_index);
        }
    }
}

// ---------------------------------------------------------------------------
// Player click — play selected card targeting self
// ---------------------------------------------------------------------------

pub fn handle_player_click(
    mut commands: Commands,
    run_data: Option<ResMut<RunData>>,
    mut targeting: ResMut<TargetingState>,
    mut hand_dirty: ResMut<HandDirty>,
    player_query: Query<&Interaction, (Changed<Interaction>, With<PlayerSpriteArea>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(mut run_data) = run_data else { return };
    let Some(hand_index) = targeting.selected_card_index else { return; };

    for interaction in &player_query {
        if *interaction != Interaction::Pressed { continue; }

        // Play the card with no enemy target (self-targeting).
        targeting.selected_card_index = None;
        let action = GauntletAction::CombatAction(Action::PlayCard {
            hand_index,
            target: None,
        });
        if let Ok(events) = run_data.runner.apply(action) {
            hand_dirty.0 = true;
            check_combat_end(&events, &mut run_data, &mut next_state, &mut commands);
        }
    }
}

// ---------------------------------------------------------------------------
// Enemy click — targeting resolution
// ---------------------------------------------------------------------------

pub fn handle_enemy_click(
    mut commands: Commands,
    run_data: Option<ResMut<RunData>>,
    mut targeting: ResMut<TargetingState>,
    mut hand_dirty: ResMut<HandDirty>,
    enemy_query: Query<(&Interaction, &EnemyTarget), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(mut run_data) = run_data else { return };
    let Some(hand_index) = targeting.selected_card_index else { return; };

    for (interaction, enemy) in &enemy_query {
        if *interaction != Interaction::Pressed { continue; }

        let action = GauntletAction::CombatAction(Action::PlayCard {
            hand_index,
            target: Some(enemy.0),
        });
        targeting.selected_card_index = None;

        if let Ok(events) = run_data.runner.apply(action) {
            hand_dirty.0 = true;
            check_combat_end(&events, &mut run_data, &mut next_state, &mut commands);
        }
    }
}

// ---------------------------------------------------------------------------
// End Turn
// ---------------------------------------------------------------------------

pub fn handle_end_turn(
    mut commands: Commands,
    run_data: Option<ResMut<RunData>>,
    mut hand_dirty: ResMut<HandDirty>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<EndTurnButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(mut run_data) = run_data else { return };

    for interaction in &button_query {
        if *interaction != Interaction::Pressed { continue; }
        let action = GauntletAction::CombatAction(Action::EndTurn);
        if let Ok(events) = run_data.runner.apply(action) {
            hand_dirty.0 = true;
            check_combat_end(&events, &mut run_data, &mut next_state, &mut commands);
        }
    }
}

// ---------------------------------------------------------------------------
// Cancel targeting
// ---------------------------------------------------------------------------

pub fn cancel_targeting(
    mut targeting: ResMut<TargetingState>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
        targeting.selected_card_index = None;
    }
}

// ---------------------------------------------------------------------------
// Check combat end
// ---------------------------------------------------------------------------

fn check_combat_end(
    events: &[GauntletEvent],
    run_data: &mut RunData,
    next_state: &mut ResMut<NextState<GameState>>,
    commands: &mut Commands,
) {
    // Check RunWon first — it comes after FightWon in the same event list.
    let run_won = events.iter().find_map(|e| {
        if let GauntletEvent::RunWon { fights_won } = e {
            Some(*fights_won)
        } else {
            None
        }
    });

    if let Some(fights_won) = run_won {
        commands.insert_resource(super::RunWon { fights_won });
        next_state.set(GameState::GameOver);
        return;
    }

    for event in events {
        match event {
            GauntletEvent::FightWon { .. } => {
                // Process level-up banners from CardGrant / PerkGrant events.
                crate::plugins::phase_routing::check_level_up(events, commands);
                // Route to whatever phase the engine is now in (Reward,
                // DeckSwap, InnateChoice, DeckRebuild, Combat, etc.)
                crate::plugins::phase_routing::route_to_phase(run_data, next_state);
                return;
            }
            GauntletEvent::FightLost { .. } => {
                next_state.set(GameState::GameOver);
                return;
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tooltip sync — keep tooltip text in sync with current game state
// ---------------------------------------------------------------------------

pub fn sync_tooltips(
    run_data: Option<Res<RunData>>,
    mut tooltip_queries: ParamSet<(
        Query<&mut TooltipContent, With<EnergyOrb>>,
        Query<&mut TooltipContent, With<PlayerBlockContainer>>,
        Query<&mut TooltipContent, With<DrawPileCard>>,
        Query<&mut TooltipContent, With<DiscardPileCard>>,
        Query<(&EnemyIntentContainer, &mut TooltipContent)>,
        Query<&mut TooltipContent, With<ExhaustPileCard>>,
    )>,
) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    // Energy orb tooltip
    if let Ok(mut tc) = tooltip_queries.p0().single_mut() {
        tc.0 = tooltip_text::energy_tooltip(obs.player_energy, obs.player_max_energy);
    }

    // Block shield tooltip
    if let Ok(mut tc) = tooltip_queries.p1().single_mut() {
        tc.0 = tooltip_text::block_tooltip(obs.player_block);
    }

    // Draw pile tooltip
    if let Ok(mut tc) = tooltip_queries.p2().single_mut() {
        tc.0 = tooltip_text::draw_pile_tooltip(obs.draw_pile_size);
    }

    // Discard pile tooltip
    if let Ok(mut tc) = tooltip_queries.p3().single_mut() {
        tc.0 = tooltip_text::discard_pile_tooltip(obs.discard_pile_size);
    }

    // Enemy intent tooltips
    for (container, mut tc) in &mut tooltip_queries.p4() {
        let idx = container.0;
        if let Some(enemy) = obs.enemies.iter().find(|e| e.index == idx) {
            tc.0 = tooltip_text::enemy_intent_tooltip(&enemy.intent);
        }
    }

    // Exhaust pile tooltip
    if let Ok(mut tc) = tooltip_queries.p5().single_mut() {
        tc.0 = tooltip_text::exhaust_pile_tooltip(obs.exhaust_pile_card_ids.len());
    }
}

// ---------------------------------------------------------------------------
// Level-up banner — fade out and remove after timer expires
// ---------------------------------------------------------------------------

pub fn tick_level_up_banner(
    mut commands: Commands,
    time: Res<Time>,
    mut banner: Option<ResMut<super::LevelUpBanner>>,
    mut banner_query: Query<(Entity, &mut BackgroundColor), (With<LevelUpBannerNode>, Without<PerkBannerNode>)>,
) {
    let Some(ref mut banner) = banner else { return };

    banner.timer.tick(time.delta());

    if banner.timer.is_finished() {
        for (entity, _) in &banner_query {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<super::LevelUpBanner>();
    } else {
        let remaining = banner.timer.remaining_secs();
        let alpha = (remaining / banner.timer.duration().as_secs_f32()).min(1.0) * 0.85;
        for (_, mut bg) in &mut banner_query {
            *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));
        }
    }
}

// ---------------------------------------------------------------------------
// Perk banner — fade out and remove after timer expires
// ---------------------------------------------------------------------------

pub fn tick_perk_banner(
    mut commands: Commands,
    time: Res<Time>,
    mut banner: Option<ResMut<super::PerkBanner>>,
    mut banner_query: Query<(Entity, &mut BackgroundColor), (With<PerkBannerNode>, Without<LevelUpBannerNode>)>,
) {
    let Some(ref mut banner) = banner else { return };

    banner.timer.tick(time.delta());

    if banner.timer.is_finished() {
        for (entity, _) in &banner_query {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<super::PerkBanner>();
    } else {
        let remaining = banner.timer.remaining_secs();
        let alpha = (remaining / banner.timer.duration().as_secs_f32()).min(1.0) * 0.85;
        for (_, mut bg) in &mut banner_query {
            *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));
        }
    }
}

// ---------------------------------------------------------------------------
// Pile viewer — click draw/discard/exhaust pile to see cards
// ---------------------------------------------------------------------------

pub fn handle_pile_click(
    mut commands: Commands,
    run_data: Option<Res<RunData>>,
    draw_query: Query<&Interaction, (Changed<Interaction>, With<DrawPileCard>)>,
    discard_query: Query<&Interaction, (Changed<Interaction>, With<DiscardPileCard>, Without<DrawPileCard>)>,
    exhaust_query: Query<&Interaction, (Changed<Interaction>, With<ExhaustPileCard>, Without<DrawPileCard>, Without<DiscardPileCard>)>,
    deck_query: Query<&Interaction, (Changed<Interaction>, With<ViewDeckButton>)>,
    existing: Query<Entity, With<PileViewerOverlay>>,
) {
    if !existing.is_empty() { return; }
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    let mut title = None;
    let mut card_ids: Vec<decker_engine::card_ids::CardId> = Vec::new();

    for interaction in &draw_query {
        if *interaction == Interaction::Pressed {
            title = Some(format!("Draw Pile ({} cards)", obs.draw_pile_card_ids.len()));
            card_ids = obs.draw_pile_card_ids.clone();
        }
    }
    for interaction in &discard_query {
        if *interaction == Interaction::Pressed {
            title = Some(format!("Discard Pile ({} cards)", obs.discard_pile_card_ids.len()));
            card_ids = obs.discard_pile_card_ids.clone();
        }
    }
    for interaction in &exhaust_query {
        if *interaction == Interaction::Pressed {
            title = Some(format!("Exhaust Pile ({} cards)", obs.exhaust_pile_card_ids.len()));
            card_ids = obs.exhaust_pile_card_ids.clone();
        }
    }
    for interaction in &deck_query {
        if *interaction == Interaction::Pressed {
            title = Some(format!("Play Deck ({} cards)", obs.play_deck_card_ids.len()));
            card_ids = obs.play_deck_card_ids.clone();
        }
    }

    let Some(title) = title else { return };

    commands.insert_resource(PileViewerJustOpened);
    commands.spawn((
        PileViewerOverlay,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        },
        GlobalZIndex(90),
        BackgroundColor(theme::OVERLAY_BLACK),
    ))
    .with_children(|overlay| {
        overlay.spawn((
            Text::new(&title),
            TextFont { font_size: 24.0, ..default() },
            TextColor(theme::GOLD),
        ));

        overlay.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::FlexStart,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            overflow: Overflow::scroll_y(),
            ..default()
        })
        .with_children(|grid| {
            let config = decker_card_renderer::CardDisplayConfig::mini();
            for card_id in &card_ids {
                if let Some(card_def) = run_data.runner.content.card_defs.get(card_id) {
                    let entity = decker_card_renderer::spawn_card(grid, card_def, &config);
                    let tooltip = tooltip_text::card_tooltip(
                        &card_def.name,
                        &card_def.description,
                        card_def.cost,
                        card_def.exhaust,
                        &[], &[],
                    );
                    grid.commands().entity(entity).insert((
                        Button,
                        TooltipContent(tooltip),
                    ));
                }
            }
            if card_ids.is_empty() {
                grid.spawn((
                    Text::new("Empty"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(theme::SLATE),
                ));
            }
        });

        overlay.spawn((
            Text::new("Click anywhere or press Escape to close"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(theme::SLATE),
            Node { flex_shrink: 0.0, ..default() },
        ));
    });
}

/// Resource to delay pile viewer close by one frame (so the opening click doesn't close it).
#[derive(Resource)]
pub struct PileViewerJustOpened;

pub fn handle_pile_viewer_close(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    overlay_query: Query<Entity, With<PileViewerOverlay>>,
    just_opened: Option<Res<PileViewerJustOpened>>,
) {
    if overlay_query.is_empty() { return; }

    // Skip one frame after opening to avoid the opening click closing it
    if just_opened.is_some() {
        commands.remove_resource::<PileViewerJustOpened>();
        return;
    }

    if keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Left) {
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
    }
}
