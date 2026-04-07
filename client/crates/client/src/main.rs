//! Short Rest — a roguelike deckbuilder (Gauntlet MVP).
//!
//! This is the Bevy application entry point. It assembles plugins, sets up the
//! window, and hands control to the state machine starting at `MainMenu`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetPlugin;
use bevy::camera::Viewport;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy::winit::WinitSettings;
use clap::Parser;

mod plugins;
mod state;
mod theme;
mod tooltip;
mod tooltip_text;

use plugins::{
    CollectionOverflowPlugin, CombatPlugin, DeckRebuildPlugin, DeckSwapPlugin,
    GameOverPlugin, GauntletRewardPlugin, InnateChoicePlugin, MainMenuPlugin, RunStartPlugin,
};
use state::GameState;

#[derive(Parser, Debug)]
#[command(name = "decker", about = "Short Rest — a roguelike deckbuilder")]
struct Cli {
    /// Pre-select character race (e.g., human)
    #[arg(long)]
    race: Option<String>,

    /// Pre-select character background (e.g., soldier)
    #[arg(long)]
    background: Option<String>,
}

/// CLI-derived defaults, inserted as a Bevy resource for the wizard to read.
#[derive(Resource)]
pub struct CliDefaults {
    pub race: Option<String>,
    pub background: Option<String>,
}

/// Reference aspect ratio (16:9). The UI is designed for this ratio.
/// At different ratios, blank space is added (letterbox or pillarbox).
const ASPECT_W: f32 = 16.0;
const ASPECT_H: f32 = 9.0;

/// Reference height in logical pixels. UI is authored at this scale;
/// actual resolution is handled by uniform scaling.
const REFERENCE_HEIGHT: f32 = 720.0;

fn main() {
    let cli = Cli::parse();

    // Support headless/container mode via DECKER_WINDOW env var.
    // e.g., DECKER_WINDOW=1280x720 forces windowed mode at that resolution.
    let (window_mode, window_resolution) = match std::env::var("DECKER_WINDOW").ok() {
        Some(spec) => {
            let parts: Vec<&str> = spec.split('x').collect();
            if let (Some(w), Some(h)) = (
                parts.first().and_then(|s| s.parse::<u32>().ok()),
                parts.get(1).and_then(|s| s.parse::<u32>().ok()),
            ) {
                (
                    WindowMode::Windowed,
                    bevy::window::WindowResolution::new(w, h),
                )
            } else {
                (
                    WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    default(),
                )
            }
        }
        None => (
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            default(),
        ),
    };

    let mut app = App::new();

    // DECKER_FPS env var: cap framerate for headless/CUA usage (e.g. DECKER_FPS=5)
    if let Ok(fps_str) = std::env::var("DECKER_FPS") {
        if let Ok(fps) = fps_str.parse::<f64>() {
            app.insert_resource(WinitSettings {
                focused_mode: bevy::winit::UpdateMode::reactive_low_power(
                    std::time::Duration::from_secs_f64(1.0 / fps),
                ),
                unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(
                    std::time::Duration::from_secs_f64(1.0 / fps),
                ),
            });
        }
    }

    app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Short Rest".into(),
                        mode: window_mode,
                        resolution: window_resolution,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: if cfg!(debug_assertions) {
                        // cargo run executes from target/debug/ — go up to workspace root.
                        "../../assets".into()
                    } else {
                        "assets".into()
                    },
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(RunStartPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(GauntletRewardPlugin)
        .add_plugins(DeckSwapPlugin)
        .add_plugins(CollectionOverflowPlugin)
        .add_plugins(InnateChoicePlugin)
        .add_plugins(DeckRebuildPlugin)
        .add_plugins(GameOverPlugin)
        .add_plugins(decker_card_renderer::CardRendererPlugin)
        .add_plugins(tooltip::TooltipPlugin)
        .insert_resource(CliDefaults {
            race: cli.race,
            background: cli.background,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (update_scaling, toggle_fullscreen));

    // DECKER_STATE_EXPORT: write observation JSON after each state change
    if std::env::var("DECKER_STATE_EXPORT").is_ok() {
        app.add_systems(Update, export_game_state);
    }

    app.run();
}

/// Writes the current game observation + full save state to files for CUA/bot consumption.
/// Only active when DECKER_STATE_EXPORT env var is set.
fn export_game_state(run_data: Option<Res<plugins::RunData>>) {
    let Some(run_data) = run_data else { return };
    if !run_data.is_changed() { return; }

    let export_path = std::env::var("DECKER_STATE_EXPORT")
        .unwrap_or_else(|_| "/tmp/decker_state.json".to_string());

    let obs = run_data.runner.observe();
    if let Ok(json) = serde_json::to_string_pretty(&obs) {
        let _ = std::fs::write(&export_path, &json);
    }

    // Write legal action labels for bot consumption
    let actions_path = format!("{}", export_path.replace(".json", "_actions.json"));
    let action_labels: Vec<String> = run_data.runner.legal_actions()
        .iter().map(|a| format!("{:?}", a)).collect();
    if let Ok(json) = serde_json::to_string_pretty(&action_labels) {
        let _ = std::fs::write(&actions_path, &json);
    }

    // Also write full save state for save/load
    let save_path = format!("{}.save", export_path.trim_end_matches(".json"));
    if let Ok(json) = serde_json::to_string_pretty(&run_data.runner) {
        let _ = std::fs::write(&save_path, &json);
    }
}

fn compute_scale_and_viewport(window: &Window) -> (f32, Viewport) {
    let win_w = window.width();
    let win_h = window.height();
    let sf = window.scale_factor();

    // Fit the largest 16:9 rect inside the window.
    let target_ratio = ASPECT_W / ASPECT_H;
    let window_ratio = win_w / win_h;

    let (vp_w, vp_h) = if window_ratio > target_ratio {
        // Window is wider than 16:9 → pillarbox (blank on sides)
        (win_h * target_ratio, win_h)
    } else {
        // Window is taller than 16:9 → letterbox (blank top/bottom)
        (win_w, win_w / target_ratio)
    };

    // Center the viewport in the window
    let offset_x = ((win_w - vp_w) * 0.5 * sf).round() as u32;
    let offset_y = ((win_h - vp_h) * 0.5 * sf).round() as u32;

    let viewport = Viewport {
        physical_position: UVec2::new(offset_x, offset_y),
        physical_size: UVec2::new(
            (vp_w * sf).max(1.0) as u32,
            (vp_h * sf).max(1.0) as u32,
        ),
        ..default()
    };

    // Scale UI so reference height maps to the viewport height
    let scale = (vp_h / REFERENCE_HEIGHT).max(0.1);

    (scale, viewport)
}

fn setup(mut commands: Commands, windows: Query<&Window>) {
    let window = windows.single().unwrap();
    let (scale, viewport) = compute_scale_and_viewport(window);
    commands.insert_resource(UiScale(scale));
    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(viewport),
            ..default()
        },
    ));
}

fn update_scaling(
    windows: Query<&Window, Changed<Window>>,
    mut ui_scale: ResMut<UiScale>,
    mut camera_query: Query<&mut Camera, With<Camera2d>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (scale, viewport) = compute_scale_and_viewport(window);
    *ui_scale = UiScale(scale);
    if let Ok(mut camera) = camera_query.single_mut() {
        camera.viewport = Some(viewport);
    }
}

fn toggle_fullscreen(keys: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>) {
    if !keys.just_pressed(KeyCode::F11) {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    window.mode = match window.mode {
        WindowMode::BorderlessFullscreen(_) => {
            let ref_w = (REFERENCE_HEIGHT * ASPECT_W / ASPECT_H) as u32;
            window.resolution = (ref_w, REFERENCE_HEIGHT as u32).into();
            WindowMode::Windowed
        }
        _ => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
    };
}
