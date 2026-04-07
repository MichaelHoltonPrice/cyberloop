//! Combat UI — renders the current fight and handles card play + targeting.
//!
//! Layout: Player character on the left, enemies on the right, hand at the
//! bottom, with a Bézier targeting arrow connecting selected card → cursor.

pub mod setup;
pub mod systems;

use bevy::prelude::*;

use crate::state::GameState;
use crate::theme;
use crate::tooltip::TooltipSet;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Combat), setup::setup_combat)
            .add_systems(
                Update,
                (
                    (
                        systems::cancel_targeting,
                        systems::handle_hand_deselect,
                        systems::handle_card_click,
                        systems::handle_player_click,
                        systems::handle_enemy_click,
                        systems::handle_end_turn,
                        systems::rebuild_hand,
                        systems::sync_combat_display,
                        systems::sync_enemy_panels,
                    ).chain(),
                    (
                        systems::swap_dead_sprites,
                        systems::swap_dead_player_sprite,
                        systems::sync_player_statuses,
                        systems::sync_enemy_statuses,
                        systems::card_hover_effects,
                        systems::update_targeting_arrow,
                        systems::update_targeting_visuals,
                        systems::sync_tooltips,
                        systems::handle_pile_click,
                        systems::handle_pile_viewer_close,
                        systems::tick_level_up_banner,
                        systems::tick_perk_banner,
                    ).chain(),
                )
                    .chain()
                    .before(TooltipSet)
                    .run_if(in_state(GameState::Combat)),
            )
            .add_systems(OnExit(GameState::Combat), setup::cleanup_combat);
    }
}

/// Resource tracking which card is selected for targeting.
#[derive(Resource, Default)]
pub struct TargetingState {
    pub selected_card_index: Option<usize>,
}

/// Cache of last-rendered status lists to avoid despawn/respawn flicker.
#[derive(Resource, Default)]
pub struct LastRenderedStatuses {
    pub player: Vec<(decker_engine::status::StatusType, i32)>,
    pub player_block_floor: i32,
    pub player_retain_cap: i32,
    pub enemies: std::collections::HashMap<usize, Vec<(decker_engine::status::StatusType, i32)>>,
}

/// Flag to trigger hand card rebuild.
#[derive(Resource)]
pub struct HandDirty(pub bool);

/// Queued level-up banner to show when combat starts.
#[derive(Resource)]
pub struct LevelUpBanner {
    pub new_max_hp: i32,
    pub healed: i32,
    pub timer: Timer,
}

/// Queued perk banner to show when combat starts.
#[derive(Resource)]
pub struct PerkBanner {
    pub message: String,
    pub timer: Timer,
}

/// Inserted when the player wins the gauntlet (50 fights).
/// Used by game_over_ui to show victory instead of defeat.
#[derive(Resource)]
pub struct RunWon {
    pub fights_won: u32,
}

/// The active combat palette for the current fight.
/// Systems that need per-background colors read this resource.
#[derive(Resource)]
pub struct CurrentCombatPalette(pub &'static theme::CombatPalette);
