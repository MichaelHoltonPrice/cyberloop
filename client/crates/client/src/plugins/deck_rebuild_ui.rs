//! Deck Rebuild screen — rebuild the 16-card play deck from the full collection.
//!
//! Single-screen layout: deck on the left, available cards on the right.
//! Inherent cards (race/background) are auto-added and locked.
//! Click an available card to add it. Click a deck card to remove it.
//! Confirm when 16/16 to apply.

use std::collections::HashMap;

use bevy::prelude::*;

use decker_engine::card_ids::CardId;
use decker_gauntlet::GauntletAction;

use crate::plugins::phase_routing;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;

pub struct DeckRebuildPlugin;

impl Plugin for DeckRebuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::DeckRebuild), setup_deck_rebuild)
            .add_systems(
                Update,
                (
                    handle_available_pick,
                    handle_deck_remove,
                    handle_confirm,
                )
                    .run_if(in_state(GameState::DeckRebuild)),
            )
            .add_systems(OnExit(GameState::DeckRebuild), cleanup_deck_rebuild);
    }
}

// ── Resource: client-side draft ──────────────────────────────────

#[derive(Resource)]
struct DeckRebuildDraft {
    /// 16 slots, Some = filled, None = empty.
    selections: Vec<Option<CardId>>,
    /// Total copies of each card available in the collection.
    pool_counts: HashMap<CardId, usize>,
    /// Ordered unique card IDs for display.
    unique_card_order: Vec<CardId>,
    /// Card IDs that are inherent (auto-added, cannot be removed).
    inherent_ids: Vec<CardId>,
}

impl DeckRebuildDraft {
    fn new(
        remaining_card_ids: &[CardId],
        current_deck_ids: &[CardId],
        card_defs: &HashMap<CardId, decker_engine::card::CardDef>,
    ) -> Self {
        let mut pool_counts: HashMap<CardId, usize> = HashMap::new();
        let mut unique_card_order = Vec::new();
        for id in remaining_card_ids {
            let count = pool_counts.entry(id.clone()).or_insert(0);
            if *count == 0 {
                unique_card_order.push(id.clone());
            }
            *count += 1;
        }

        // Find inherent cards
        let inherent_ids: Vec<CardId> = unique_card_order
            .iter()
            .filter(|id| {
                card_defs
                    .get(id.as_str())
                    .is_some_and(|d| d.inherent)
            })
            .cloned()
            .collect();

        // Pre-populate with current deck (if cards are in the pool)
        let mut selections: Vec<Option<CardId>> = vec![None; 16];
        let mut temp_pool = pool_counts.clone();
        for (i, card_id) in current_deck_ids.iter().enumerate() {
            if i >= 16 { break; }
            if let Some(count) = temp_pool.get_mut(card_id) {
                if *count > 0 {
                    selections[i] = Some(card_id.clone());
                    *count -= 1;
                }
            }
        }

        Self {
            selections,
            pool_counts,
            unique_card_order,
            inherent_ids,
        }
    }

    fn available_count(&self, card_id: &CardId) -> usize {
        let total = self.pool_counts.get(card_id).copied().unwrap_or(0);
        let used = self
            .selections
            .iter()
            .filter(|s| s.as_ref() == Some(card_id))
            .count();
        total.saturating_sub(used)
    }

    fn filled_count(&self) -> usize {
        self.selections.iter().filter(|s| s.is_some()).count()
    }

    fn is_complete(&self) -> bool {
        self.selections.iter().all(|s| s.is_some())
    }

    fn next_empty_slot(&self) -> Option<usize> {
        self.selections.iter().position(|s| s.is_none())
    }

    fn is_inherent(&self, card_id: &CardId) -> bool {
        self.inherent_ids.contains(card_id)
    }
}

// ── Components ───────────────────────────────────────────────────

#[derive(Component)] struct DeckRebuildRoot;
#[derive(Component)] struct DeckGrid;
#[derive(Component)] struct AvailableGrid;
#[derive(Component)] struct ProgressText;
#[derive(Component)] struct ConfirmButton;
#[derive(Component)] struct ConfirmButtonText;

/// A card in the deck panel (click to remove, unless inherent).
#[derive(Component)] struct DeckSlot(usize);

/// An available card (click to add to deck).
#[derive(Component)] struct AvailablePick(usize);

// ── Setup ────────────────────────────────────────────────────────

fn setup_deck_rebuild(mut commands: Commands, run_data: Option<Res<RunData>>) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    let draft = DeckRebuildDraft::new(
        &obs.rebuild_remaining_card_ids,
        &obs.play_deck_card_ids,
        &run_data.runner.content.card_defs,
    );

    commands
        .spawn((
            DeckRebuildRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("REST STOP - REBUILD YOUR DECK"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(theme::TEAL),
                Node { flex_shrink: 0.0, ..default() },
            ));

            // Progress + HP on one line
            root.spawn((
                ProgressText,
                Text::new(format!(
                    "{}/16 cards selected  |  HP: {}/{}",
                    draft.filled_count(), obs.player_hp, obs.player_max_hp,
                )),
                TextFont { font_size: 14.0, ..default() },
                TextColor(theme::GOLD),
                Node { flex_shrink: 0.0, ..default() },
            ));

            // Main content: two columns (takes all remaining space)
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                overflow: Overflow::clip_y(),
                ..default()
            })
            .with_children(|columns| {
                // Left: Your Deck
                columns.spawn(Node {
                    width: Val::Percent(45.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                })
                .with_children(|left| {
                    left.spawn((
                        Text::new("Your Deck (click to remove)"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(theme::PARCHMENT),
                    ));

                    left.spawn((
                        DeckGrid,
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::FlexStart,
                            column_gap: Val::Px(4.0),
                            row_gap: Val::Px(4.0),
                            flex_grow: 1.0,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                    ))
                    .with_children(|grid| {
                        spawn_deck_cards(grid, &draft, &run_data);
                    });
                });

                // Separator
                columns.spawn((
                    Node {
                        width: Val::Px(1.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(theme::BORDER_SLATE),
                ));

                // Right: Available Cards
                columns.spawn(Node {
                    width: Val::Percent(55.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                })
                .with_children(|right| {
                    right.spawn((
                        Text::new("Available Cards (click to add)"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(theme::PARCHMENT),
                    ));

                    right.spawn((
                        AvailableGrid,
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::FlexStart,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            flex_grow: 1.0,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                    ))
                    .with_children(|grid| {
                        spawn_available_cards(grid, &draft, &run_data);
                    });
                });
            });

            // Confirm button
            let is_ready = draft.is_complete();
            root.spawn((
                ConfirmButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(32.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(if is_ready { theme::GOLD } else { theme::BORDER_SLATE }),
            ))
            .with_child((
                ConfirmButtonText,
                Text::new("Confirm Rebuild"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(if is_ready { theme::GOLD } else { theme::SLATE }),
            ));
        });

    commands.insert_resource(draft);
}

// ── Spawn helpers ────────────────────────────────────────────────

fn spawn_deck_cards(
    parent: &mut ChildSpawnerCommands,
    draft: &DeckRebuildDraft,
    run_data: &RunData,
) {
    let config = decker_card_renderer::CardDisplayConfig::mini();
    let w = decker_card_renderer::card_width(&config);
    let h = decker_card_renderer::card_height(&config);

    for (i, selection) in draft.selections.iter().enumerate() {
        if let Some(card_id) = selection {
            let is_locked = draft.is_inherent(card_id);
            let card_config = if is_locked {
                config.clone().with_dimmed(true)
            } else {
                config.clone()
            };
            if let Some(card_def) = run_data.runner.content.card_defs.get(card_id) {
                let entity = decker_card_renderer::spawn_card(parent, card_def, &card_config);
                if !is_locked {
                    parent.commands().entity(entity).insert((Button, DeckSlot(i)));
                }
            }
        } else {
            // Empty slot
            parent.spawn((
                Node {
                    width: Val::Px(w),
                    height: Val::Px(h),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BORDER_SLATE),
            ))
            .with_child((
                Text::new(format!("{}", i + 1)),
                TextFont { font_size: 10.0, ..default() },
                TextColor(theme::SLATE),
            ));
        }
    }
}

fn spawn_available_cards(
    parent: &mut ChildSpawnerCommands,
    draft: &DeckRebuildDraft,
    run_data: &RunData,
) {
    let config = decker_card_renderer::CardDisplayConfig::mini();
    for (i, card_id) in draft.unique_card_order.iter().enumerate() {
        let available = draft.available_count(card_id);
        if available == 0 { continue; }

        // Don't show inherent cards in available (they're auto-added)
        if draft.is_inherent(card_id) { continue; }

        if let Some(card_def) = run_data.runner.content.card_defs.get(card_id) {
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|col| {
                let entity = decker_card_renderer::spawn_card(col, card_def, &config);
                col.commands().entity(entity).insert((Button, AvailablePick(i)));

                if available > 1 {
                    col.spawn((
                        Text::new(format!("x{available}")),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(theme::SLATE),
                    ));
                }
            });
        }
    }
}

// ── Refresh helper ───────────────────────────────────────────────

fn refresh_ui(
    commands: &mut Commands,
    draft: &DeckRebuildDraft,
    run_data: &RunData,
    progress: &mut Query<&mut Text, (With<ProgressText>, Without<ConfirmButtonText>)>,
    deck_grid: &Query<Entity, With<DeckGrid>>,
    avail_grid: &Query<Entity, (With<AvailableGrid>, Without<DeckGrid>)>,
    confirm_border: &mut Query<&mut BorderColor, With<ConfirmButton>>,
    confirm_text: &mut Query<&mut TextColor, With<ConfirmButtonText>>,
) {
    if let Ok(mut text) = progress.single_mut() {
        **text = format!("{}/16 cards selected", draft.filled_count());
    }

    if let Ok(entity) = deck_grid.single() {
        commands.entity(entity).despawn_children();
        commands.entity(entity).with_children(|grid| {
            spawn_deck_cards(grid, draft, run_data);
        });
    }

    if let Ok(entity) = avail_grid.single() {
        commands.entity(entity).despawn_children();
        commands.entity(entity).with_children(|grid| {
            spawn_available_cards(grid, draft, run_data);
        });
    }

    let is_ready = draft.is_complete();
    if let Ok(mut border) = confirm_border.single_mut() {
        *border = BorderColor::all(if is_ready { theme::GOLD } else { theme::BORDER_SLATE });
    }
    if let Ok(mut color) = confirm_text.single_mut() {
        *color = TextColor(if is_ready { theme::GOLD } else { theme::SLATE });
    }
}

// ── Systems ──────────────────────────────────────────────────────

/// Click an available card → add to next empty deck slot.
fn handle_available_pick(
    mut commands: Commands,
    mut draft: Option<ResMut<DeckRebuildDraft>>,
    run_data: Option<Res<RunData>>,
    query: Query<(&Interaction, &AvailablePick), (Changed<Interaction>, With<Button>)>,
    mut progress: Query<&mut Text, (With<ProgressText>, Without<ConfirmButtonText>)>,
    deck_grid: Query<Entity, With<DeckGrid>>,
    avail_grid: Query<Entity, (With<AvailableGrid>, Without<DeckGrid>)>,
    mut confirm_border: Query<&mut BorderColor, With<ConfirmButton>>,
    mut confirm_text: Query<&mut TextColor, With<ConfirmButtonText>>,
) {
    let Some(ref mut draft) = draft else { return };
    let Some(ref run_data) = run_data else { return };

    for (interaction, pick) in &query {
        if *interaction != Interaction::Pressed { continue; }

        let Some(slot_idx) = draft.next_empty_slot() else { continue; };
        let card_id = draft.unique_card_order[pick.0].clone();
        if draft.available_count(&card_id) == 0 { continue; }

        draft.selections[slot_idx] = Some(card_id);
        refresh_ui(
            &mut commands, draft, run_data,
            &mut progress, &deck_grid, &avail_grid,
            &mut confirm_border, &mut confirm_text,
        );
    }
}

/// Click a deck card → remove it (unless inherent).
fn handle_deck_remove(
    mut commands: Commands,
    mut draft: Option<ResMut<DeckRebuildDraft>>,
    run_data: Option<Res<RunData>>,
    query: Query<(&Interaction, &DeckSlot), (Changed<Interaction>, With<Button>)>,
    mut progress: Query<&mut Text, (With<ProgressText>, Without<ConfirmButtonText>)>,
    deck_grid: Query<Entity, With<DeckGrid>>,
    avail_grid: Query<Entity, (With<AvailableGrid>, Without<DeckGrid>)>,
    mut confirm_border: Query<&mut BorderColor, With<ConfirmButton>>,
    mut confirm_text: Query<&mut TextColor, With<ConfirmButtonText>>,
) {
    let Some(ref mut draft) = draft else { return };
    let Some(ref run_data) = run_data else { return };

    for (interaction, slot) in &query {
        if *interaction != Interaction::Pressed { continue; }

        if let Some(ref card_id) = draft.selections[slot.0] {
            if draft.is_inherent(card_id) { continue; }
        }

        draft.selections[slot.0] = None;
        refresh_ui(
            &mut commands, draft, run_data,
            &mut progress, &deck_grid, &avail_grid,
            &mut confirm_border, &mut confirm_text,
        );
    }
}

/// Confirm button → apply all selections to engine → next phase.
///
/// The engine forces inherent cards to be placed first, so we can't
/// submit our selections in our slot order. Instead, for each of the
/// 16 engine steps, we check what the engine offers and pick the best
/// match from our desired selections.
fn handle_confirm(
    mut commands: Commands,
    draft: Option<Res<DeckRebuildDraft>>,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ConfirmButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref draft) = draft else { return };
    let Some(ref mut run_data) = run_data else { return };

    if !draft.is_complete() { return; }

    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }

        // Build a bag of desired card IDs (with counts for duplicates)
        let mut desired: HashMap<CardId, usize> = HashMap::new();
        for selection in &draft.selections {
            if let Some(card_id) = selection {
                *desired.entry(card_id.clone()).or_insert(0) += 1;
            }
        }

        // Submit 16 cards, respecting the engine's forced ordering
        for _ in 0..16 {
            let obs = run_data.runner.observe();
            if obs.choice_cards.is_empty() { break; }

            // Find the first offered card that we still want
            let pick = obs.choice_cards.iter().enumerate().find(|(_, c)| {
                desired.get(&c.card_id).copied().unwrap_or(0) > 0
            });

            if let Some((choice_idx, card_obs)) = pick {
                if let Some(count) = desired.get_mut(&card_obs.card_id) {
                    *count = count.saturating_sub(1);
                }
                if let Ok(events) =
                    run_data.runner.apply(GauntletAction::ChooseDeckSlotCard(choice_idx))
                {
                    phase_routing::check_level_up(&events, &mut commands);
                }
            } else {
                // Fallback: pick the first available card
                if let Ok(events) =
                    run_data.runner.apply(GauntletAction::ChooseDeckSlotCard(0))
                {
                    phase_routing::check_level_up(&events, &mut commands);
                }
            }
        }

        phase_routing::route_to_phase(run_data.as_ref(), &mut next_state);
        return;
    }
}

// ── Cleanup ──────────────────────────────────────────────────────

fn cleanup_deck_rebuild(
    mut commands: Commands,
    root_query: Query<Entity, With<DeckRebuildRoot>>,
) {
    for entity in &root_query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DeckRebuildDraft>();
}
