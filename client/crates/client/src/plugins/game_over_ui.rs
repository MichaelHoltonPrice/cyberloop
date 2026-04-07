//! Game Over screen — shows defeat, score, and navigation options.

use bevy::prelude::*;

use crate::plugins::RunWon;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;
use crate::theme;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(
                Update,
                handle_game_over_buttons.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over);
    }
}

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
struct NewRunButton;

#[derive(Component)]
struct MainMenuButton;

fn setup_game_over(
    mut commands: Commands,
    run_data: Option<Res<RunData>>,
    run_won: Option<Res<RunWon>>,
) {
    let (fights_won, subclass) = run_data
        .map(|rd| (rd.runner.fights_won, rd.runner.subclass_id.clone()))
        .unwrap_or((0, "unknown".into()));

    let is_victory = run_won.is_some();

    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // Header — VICTORY or DEFEAT
            if is_victory {
                root.spawn((
                    Text::new("VICTORY"),
                    TextFont { font_size: 64.0, ..default() },
                    TextColor(theme::GOLD),
                ));
                root.spawn((
                    Text::new("You conquered the gauntlet!"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(theme::PARCHMENT),
                ));
            } else {
                root.spawn((
                    Text::new("DEFEAT"),
                    TextFont { font_size: 64.0, ..default() },
                    TextColor(theme::HEALTH_RED),
                ));
            }

            // Score
            root.spawn((
                Text::new(format!("Fights Won: {}", fights_won)),
                TextFont { font_size: 28.0, ..default() },
                TextColor(theme::GOLD),
            ));

            // Sub-class played
            root.spawn((
                Text::new(format!("Sub-class: {}", subclass)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            // Buttons row
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                margin: UiRect::top(Val::Px(40.0)),
                ..default()
            })
            .with_children(|row| {
                // New Run
                row.spawn((
                    NewRunButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(32.0), Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL),
                    BorderColor::all(theme::GOLD),
                ))
                .with_child((
                    Text::new("New Run"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(theme::GOLD),
                ));

                // Main Menu
                row.spawn((
                    MainMenuButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(32.0), Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL),
                    BorderColor::all(theme::BORDER_SLATE),
                ))
                .with_child((
                    Text::new("Main Menu"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(theme::PARCHMENT),
                ));
            });
        });
}

fn handle_game_over_buttons(
    mut commands: Commands,
    new_run_query: Query<&Interaction, (Changed<Interaction>, With<NewRunButton>)>,
    main_menu_query: Query<&Interaction, (Changed<Interaction>, With<MainMenuButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &new_run_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<RunData>();
            commands.remove_resource::<RunWon>();
            next_state.set(GameState::RunStart);
        }
    }
    for interaction in &main_menu_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<RunData>();
            commands.remove_resource::<RunWon>();
            next_state.set(GameState::MainMenu);
        }
    }
}

fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
