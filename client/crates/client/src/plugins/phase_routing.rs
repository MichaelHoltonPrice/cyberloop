//! Shared helpers for post-fight phase routing.
//!
//! Every screen that applies a `GauntletAction` should call `route_to_phase`
//! afterward to transition to whatever phase the engine is now in.

use bevy::prelude::*;

use decker_gauntlet::{GauntletEvent, PerkType};
use decker_gauntlet::observation::ObservedPhase;

use crate::plugins::combat_ui::LevelUpBanner;
use crate::plugins::run_start_ui::RunData;
use crate::state::GameState;

/// Read the engine's current phase and set the matching `GameState`.
pub fn route_to_phase(run_data: &RunData, next_state: &mut NextState<GameState>) {
    let obs = run_data.runner.observe();
    match obs.phase_type {
        ObservedPhase::Combat            => next_state.set(GameState::Combat),
        ObservedPhase::Reward            => next_state.set(GameState::GauntletReward),
        ObservedPhase::DeckSwap          => next_state.set(GameState::DeckSwap),
        ObservedPhase::CollectionOverflow => next_state.set(GameState::CollectionOverflow),
        ObservedPhase::InnateChoice      => next_state.set(GameState::InnateChoice),
        ObservedPhase::DeckRebuild       => next_state.set(GameState::DeckRebuild),
        ObservedPhase::GameOver          => next_state.set(GameState::GameOver),
    }
}

/// Scan gauntlet events for LevelUp, PerkGranted, RestHealed, or CardGranted
/// and insert appropriate banner resources.
pub fn check_level_up(events: &[GauntletEvent], commands: &mut Commands) {
    for event in events {
        match event {
            GauntletEvent::LevelUp { new_max_hp, .. } => {
                commands.insert_resource(LevelUpBanner {
                    new_max_hp: *new_max_hp,
                    healed: 0,
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                });
            }
            GauntletEvent::PerkGranted { perk } => {
                let msg = match perk {
                    PerkType::DrawBonus => "PERK: Draw Bonus\n+1 card drawn per turn",
                    PerkType::InnateChoice => "PERK: Innate Choice\nChoose a card to always draw on turn 1",
                    PerkType::EnergyBonus => "PERK: Energy Bonus\n+1 max energy per turn",
                };
                commands.insert_resource(super::combat_ui::PerkBanner {
                    message: msg.into(),
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                });
            }
            GauntletEvent::RestHealed { amount } => {
                commands.insert_resource(super::combat_ui::PerkBanner {
                    message: format!("REST STOP\nHealed {} HP", amount),
                    timer: Timer::from_seconds(2.5, TimerMode::Once),
                });
            }
            GauntletEvent::CardGranted { card_id } => {
                // Title-case the card ID for display
                let name: String = card_id.split('_')
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().to_string() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                commands.insert_resource(super::combat_ui::PerkBanner {
                    message: format!("NEW CARD\n{} added to collection", name),
                    timer: Timer::from_seconds(2.5, TimerMode::Once),
                });
            }
            _ => {}
        }
    }
}
