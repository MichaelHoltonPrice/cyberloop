//! Gauntlet Reward screen — pick a card, remove a card, or skip.

use bevy::prelude::*;

use decker_gauntlet::GauntletAction;

use crate::plugins::phase_routing;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;
use crate::tooltip::TooltipContent;
use crate::tooltip_text;

pub struct GauntletRewardPlugin;

impl Plugin for GauntletRewardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GauntletReward), setup_reward)
            .add_systems(
                Update,
                (handle_reward_pick, handle_skip)
                    .run_if(in_state(GameState::GauntletReward)),
            )
            .add_systems(OnExit(GameState::GauntletReward), cleanup_reward);
    }
}

// --- Components ---

#[derive(Component)]
struct RewardRoot;

#[derive(Component)]
struct RewardCardButton(usize);

#[derive(Component)]
struct SkipButton;


// --- Setup ---

fn setup_reward(mut commands: Commands, run_data: Option<Res<RunData>>) {
    let Some(run_data) = run_data else { return };
    let obs = run_data.runner.observe();

    commands
        .spawn((
            RewardRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("CHOOSE YOUR REWARD"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(theme::GOLD),
            ));

            // Stats line
            root.spawn((
                Text::new(format!(
                    "Fight {} completed  |  Deck: {} cards  |  Collection: {}/40",
                    obs.fights_won, obs.play_deck_size, obs.collection_size
                )),
                TextFont { font_size: 16.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            // Reward cards row
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(20.0),
                ..default()
            })
            .with_children(|row| {
                for (i, card_obs) in obs.reward_cards.iter().enumerate() {
                    if let Some(card_def) = run_data.runner.content.card_defs.get(&card_obs.card_id) {
                        let config = decker_card_renderer::CardDisplayConfig::reward();
                        let entity = decker_card_renderer::spawn_card(row, card_def, &config);
                        let tooltip = tooltip_text::card_tooltip(
                            &card_obs.name,
                            &card_obs.description,
                            card_obs.cost,
                            card_obs.exhaust,
                            &card_obs.enemy_statuses,
                            &card_obs.self_statuses,
                        );
                        row.commands().entity(entity).insert((
                            Button,
                            RewardCardButton(i),
                            TooltipContent(tooltip),
                        ));
                    } else {
                        // Fallback text card
                        row.spawn((
                            RewardCardButton(i),
                            Button,
                            Node {
                                width: Val::Px(160.0),
                                min_height: Val::Px(220.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect::all(Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(theme::PANEL),
                            BorderColor::all(theme::GOLD),
                        ))
                        .with_children(|card_node| {
                            card_node.spawn((
                                Text::new(format!("{}", card_obs.cost)),
                                TextFont { font_size: 20.0, ..default() },
                                TextColor(theme::GOLD),
                            ));
                            card_node.spawn((
                                Text::new(&card_obs.name),
                                TextFont { font_size: 18.0, ..default() },
                                TextColor(theme::PARCHMENT),
                            ));
                            card_node.spawn((
                                Text::new(&card_obs.description),
                                TextFont { font_size: 12.0, ..default() },
                                TextColor(theme::PARCHMENT),
                            ));
                        });
                    }
                }
            });

            // Action buttons row
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                ..default()
            })
            .with_children(|row| {
                // Skip button
                row.spawn((
                    SkipButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL),
                    BorderColor::all(theme::GOLD),
                ))
                .with_child((
                    Text::new("Skip"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(theme::GOLD),
                ));
            });
        });
}

// --- Systems ---

fn handle_reward_pick(
    mut commands: Commands,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<(&Interaction, &RewardCardButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut run_data) = run_data else { return };

    for (interaction, reward) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Ok(events) = run_data.runner.apply(GauntletAction::PickReward(reward.0)) {
            phase_routing::check_level_up(&events, &mut commands);
            phase_routing::route_to_phase(run_data, &mut next_state);
        }
    }
}

fn handle_skip(
    mut commands: Commands,
    mut run_data: Option<ResMut<RunData>>,
    query: Query<&Interaction, (Changed<Interaction>, With<SkipButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(ref mut run_data) = run_data else { return };

    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Ok(events) = run_data.runner.apply(GauntletAction::SkipReward) {
            phase_routing::check_level_up(&events, &mut commands);
            phase_routing::route_to_phase(run_data, &mut next_state);
        }
    }
}

// --- Cleanup ---

fn cleanup_reward(mut commands: Commands, query: Query<Entity, With<RewardRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
