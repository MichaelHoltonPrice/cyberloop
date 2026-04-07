//! Innate Choice screen — pick a card to always draw on turn 1.
//!
//! Shown after the InnateChoice perk grant (fight 28).

use bevy::prelude::*;

use decker_gauntlet::GauntletAction;

use crate::plugins::phase_routing;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;

pub struct InnateChoicePlugin;

impl Plugin for InnateChoicePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InnateChoice), setup_innate_choice)
            .add_systems(
                Update,
                handle_innate_pick.run_if(in_state(GameState::InnateChoice)),
            )
            .add_systems(OnExit(GameState::InnateChoice), cleanup_innate_choice);
    }
}

// --- Components ---

#[derive(Component)]
struct InnateChoiceRoot;

#[derive(Component)]
struct InnateCardButton(usize);

// --- Setup ---

fn setup_innate_choice(mut commands: Commands, run_data: Option<Res<RunData>>) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    commands
        .spawn((
            InnateChoiceRoot,
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
                Text::new("CHOOSE AN INNATE CARD"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(theme::GOLD),
            ));

            // Subtitle
            root.spawn((
                Text::new("This card will always be in your opening hand"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            root.spawn(Node { height: Val::Px(8.0), ..default() });

            // Deck grid: 2 rows × 8 cols
            let config = decker_card_renderer::CardDisplayConfig::removal();
            let dimmed_config = decker_card_renderer::CardDisplayConfig::removal().with_dimmed(true);

            for row_start in [0usize, 8] {
                root.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    for i in row_start..row_start + 8 {
                        if i >= obs.play_deck_card_ids.len() { break; }
                        let card_id = &obs.play_deck_card_ids[i];
                        let already_innate = obs.innate_card_ids.contains(card_id);

                        if let Some(card_def) = run_data.runner.content.card_defs.get(card_id) {
                            let cfg = if already_innate { &dimmed_config } else { &config };
                            let entity = decker_card_renderer::spawn_card(row, card_def, cfg);
                            if !already_innate {
                                row.commands().entity(entity).insert((
                                    Button,
                                    InnateCardButton(i),
                                ));
                            }
                        }
                    }
                });
            }

            // Note about already-innate cards
            if !obs.innate_card_ids.is_empty() {
                root.spawn(Node { height: Val::Px(4.0), ..default() });
                root.spawn((
                    Text::new("(Dimmed cards are already innate)"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(theme::SLATE),
                ));
            }
        });
}

// --- Systems ---

fn handle_innate_pick(
    mut commands: Commands,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<(&Interaction, &InnateCardButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut run_data) = run_data else { return };

    for (interaction, card) in &query {
        if *interaction != Interaction::Pressed { continue; }
        if let Ok(events) = run_data.runner.apply(GauntletAction::ChooseInnate(card.0)) {
            phase_routing::check_level_up(&events, &mut commands);
            phase_routing::route_to_phase(run_data, &mut next_state);
        }
    }
}

// --- Cleanup ---

fn cleanup_innate_choice(mut commands: Commands, query: Query<Entity, With<InnateChoiceRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
