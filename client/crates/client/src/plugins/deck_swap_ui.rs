//! Deck Swap screen — swap a newly acquired card into the play deck or skip.
//!
//! Shown after CardGrant events (fights 4, 8, 12, 20, 24, 32, 40, 44).

use bevy::prelude::*;

use decker_gauntlet::GauntletAction;

use crate::plugins::phase_routing;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;

pub struct DeckSwapPlugin;

impl Plugin for DeckSwapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::DeckSwap), setup_deck_swap)
            .add_systems(
                Update,
                (handle_swap_slot, handle_skip_swap)
                    .run_if(in_state(GameState::DeckSwap)),
            )
            .add_systems(OnExit(GameState::DeckSwap), cleanup_deck_swap);
    }
}

// --- Components ---

#[derive(Component)]
struct DeckSwapRoot;

#[derive(Component)]
struct DeckSlotButton(usize);

#[derive(Component)]
struct SkipSwapButton;

// --- Setup ---

fn setup_deck_swap(mut commands: Commands, run_data: Option<Res<RunData>>) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    commands
        .spawn((
            DeckSwapRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("NEW CARD ACQUIRED"),
                TextFont { font_size: 28.0, ..default() },
                TextColor(theme::GOLD),
            ));

            // Subtitle
            root.spawn((
                Text::new("Swap into your deck or skip"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            // The acquired card (smaller — combat_hand scale)
            if let Some(card_obs) = &obs.acquired_card {
                if let Some(card_def) = run_data.runner.content.card_defs.get(&card_obs.card_id) {
                    let config = decker_card_renderer::CardDisplayConfig::combat_hand();
                    root.spawn(Node {
                        padding: UiRect::all(Val::Px(3.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    })
                    .insert(BorderColor::all(theme::TEAL))
                    .with_children(|wrapper| {
                        decker_card_renderer::spawn_card(wrapper, card_def, &config);
                    });
                }
            }

            // Label
            root.spawn((
                Text::new("YOUR DECK - click a slot to swap"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            // Deck grid: scrollable container with 2 rows × 8 cols
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                overflow: Overflow::scroll_y(),
                ..default()
            })
            .with_children(|scroll| {
                let config = decker_card_renderer::CardDisplayConfig::removal();
                for row_start in [0usize, 8] {
                    scroll.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|row| {
                        for i in row_start..row_start + 8 {
                            if i >= obs.play_deck_card_ids.len() { break; }
                            let card_id = &obs.play_deck_card_ids[i];
                            if let Some(card_def) = run_data.runner.content.card_defs.get(card_id) {
                                let entity = decker_card_renderer::spawn_card(row, card_def, &config);
                                row.commands().entity(entity).insert((
                                    Button,
                                    DeckSlotButton(i),
                                ));
                            }
                        }
                    });
                }
            });

            // Skip button
            root.spawn((
                SkipSwapButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    margin: UiRect::top(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BORDER_SLATE),
            ))
            .with_child((
                Text::new("Skip"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(theme::SLATE),
            ));
        });
}

// --- Systems ---

fn handle_swap_slot(
    mut commands: Commands,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<(&Interaction, &DeckSlotButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut run_data) = run_data else { return };

    for (interaction, slot) in &query {
        if *interaction != Interaction::Pressed { continue; }
        if let Ok(events) = run_data.runner.apply(GauntletAction::SwapIntoDeck(slot.0)) {
            phase_routing::check_level_up(&events, &mut commands);
            phase_routing::route_to_phase(run_data, &mut next_state);
        }
    }
}

fn handle_skip_swap(
    mut commands: Commands,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<&Interaction, (Changed<Interaction>, With<SkipSwapButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut run_data) = run_data else { return };

    for interaction in &query {
        if *interaction != Interaction::Pressed { continue; }
        if let Ok(events) = run_data.runner.apply(GauntletAction::SkipSwap) {
            phase_routing::check_level_up(&events, &mut commands);
            phase_routing::route_to_phase(run_data, &mut next_state);
        }
    }
}

// --- Cleanup ---

fn cleanup_deck_swap(mut commands: Commands, query: Query<Entity, With<DeckSwapRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
