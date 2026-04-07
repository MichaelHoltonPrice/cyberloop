//! Main Menu — title screen with "New Gauntlet" button.

use bevy::prelude::*;

use crate::state::GameState;
use crate::theme;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_menu)
            .add_systems(Update, handle_menu_buttons.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu);
    }
}

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct NewGauntletButton;

fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            // Title
            root.spawn((
                Text::new("SHORT REST"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(theme::GOLD),
            ));

            // Subtitle
            root.spawn((
                Text::new("A Roguelike Deckbuilder"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(theme::PARCHMENT),
            ));

            // Engine version
            let version = decker_content::engine_version();
            root.spawn((
                Text::new(format!("Engine {}", version)),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(theme::SLATE),
            ));

            // New Gauntlet button
            root.spawn((
                NewGauntletButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(40.0), Val::Px(16.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::GOLD),
            ))
            .with_child((
                Text::new("New Gauntlet"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(theme::GOLD),
            ));
        });
}

fn handle_menu_buttons(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<NewGauntletButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::RunStart);
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
