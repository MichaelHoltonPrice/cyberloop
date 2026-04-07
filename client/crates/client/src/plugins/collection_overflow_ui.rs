//! Collection Overflow screen — remove a card when the collection exceeds max.
//!
//! Triggered after PickReward or CardGrant when collection > 40 cards.

use bevy::prelude::*;

use decker_gauntlet::GauntletAction;

use crate::plugins::phase_routing;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;

pub struct CollectionOverflowPlugin;

impl Plugin for CollectionOverflowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::CollectionOverflow), setup_overflow)
            .add_systems(
                Update,
                handle_overflow_remove.run_if(in_state(GameState::CollectionOverflow)),
            )
            .add_systems(OnExit(GameState::CollectionOverflow), cleanup_overflow);
    }
}

// --- Components ---

#[derive(Component)]
struct OverflowRoot;

#[derive(Component)]
struct OverflowCardButton(usize);

// --- Setup ---

fn setup_overflow(mut commands: Commands, run_data: Option<Res<RunData>>) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    commands
        .spawn((
            OverflowRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("COLLECTION FULL"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(theme::HEALTH_RED),
            ));

            // Subtitle
            root.spawn((
                Text::new(format!(
                    "Remove one card to make room ({}/40)",
                    obs.collection_size
                )),
                TextFont { font_size: 16.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            root.spawn(Node { height: Val::Px(8.0), ..default() });

            // Card grid — wrapping rows of removable cards
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|grid| {
                let config = decker_card_renderer::CardDisplayConfig::removal();
                for (i, card_obs) in obs.choice_cards.iter().enumerate() {
                    if let Some(card_def) = run_data.runner.content.card_defs.get(&card_obs.card_id) {
                        let entity = decker_card_renderer::spawn_card(grid, card_def, &config);
                        grid.commands().entity(entity).insert((
                            Button,
                            OverflowCardButton(i),
                        ));
                    }
                }
            });
        });
}

// --- Systems ---

fn handle_overflow_remove(
    mut commands: Commands,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<(&Interaction, &OverflowCardButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut run_data) = run_data else { return };

    for (interaction, card) in &query {
        if *interaction != Interaction::Pressed { continue; }
        if let Ok(events) = run_data.runner.apply(GauntletAction::RemoveCollectionCard(card.0)) {
            phase_routing::check_level_up(&events, &mut commands);
            phase_routing::route_to_phase(run_data, &mut next_state);
        }
    }
}

// --- Cleanup ---

fn cleanup_overflow(mut commands: Commands, query: Query<Entity, With<OverflowRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
