//! Starter deck builders for each Fighter sub-class.
//!
//! Cyberloop: stripped to Fighter sub-classes only.

use decker_engine::card::CardInstance;
use decker_engine::card_ids::*;

use crate::cards::{background_card_id, race_card_id};

/// Build the starter deck for a given Fighter sub-class, race, and background.
///
/// All fighters get:
/// - 6 Strike + 7 Defend (universal core)
/// - 1 Second Wind (Fighter class card)
/// - 1 Race card (inherent)
/// - 1 Background card (inherent)
/// - Total: 16 cards
pub fn fighter_starter_deck(subclass_id: &str, race: &str, background: &str) -> Vec<CardInstance> {
    // Validate subclass exists (keep as a sanity check).
    match subclass_id {
        "dueling" | "defense" | "two_handed" => {}
        _ => panic!("unknown Fighter sub-class: {}", subclass_id),
    }

    let mut cards = Vec::new();

    // Universal core (6 + 7 = 13 to make room for background card)
    for _ in 0..6 {
        cards.push(CardInstance {
            def_id: STRIKE.into(),
            upgraded: false,
        });
    }
    for _ in 0..7 {
        cards.push(CardInstance {
            def_id: DEFEND.into(),
            upgraded: false,
        });
    }

    // Class card
    cards.push(CardInstance {
        def_id: SECOND_WIND.into(),
        upgraded: false,
    });

    // Race card (inherent)
    cards.push(CardInstance {
        def_id: race_card_id(race).into(),
        upgraded: false,
    });

    // Background card (inherent)
    cards.push(CardInstance {
        def_id: background_card_id(background).into(),
        upgraded: false,
    });

    cards
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::all_card_defs;

    #[test]
    fn starter_deck_size() {
        for subclass in &["two_handed", "defense", "dueling"] {
            let deck = fighter_starter_deck(subclass, "human", "soldier");
            assert_eq!(
                deck.len(),
                16,
                "{} starter deck should have 16 cards",
                subclass
            );
        }
    }

    #[test]
    fn starter_deck_all_ids_valid() {
        let defs = all_card_defs();
        for subclass in &["two_handed", "defense", "dueling"] {
            let deck = fighter_starter_deck(subclass, "human", "soldier");
            for card in &deck {
                assert!(
                    defs.contains_key(&card.def_id),
                    "{} starter deck contains unknown card {:?}",
                    subclass,
                    card.def_id
                );
            }
        }
    }

    #[test]
    fn human_has_improvise() {
        let deck = fighter_starter_deck("dueling", "human", "soldier");
        let count = deck.iter().filter(|c| c.def_id == IMPROVISE).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn soldier_has_battle_discipline() {
        let deck = fighter_starter_deck("dueling", "human", "soldier");
        let count = deck.iter().filter(|c| c.def_id == BATTLE_DISCIPLINE).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn each_race_produces_valid_deck() {
        let races = [
            "human", "high_elf", "dark_elf", "dwarf", "gnome",
            "halfling", "orc", "goblin", "dragonkin", "pantheran",
        ];
        let defs = all_card_defs();
        for race in &races {
            let deck = fighter_starter_deck("dueling", race, "soldier");
            assert_eq!(deck.len(), 16, "race {} deck size", race);
            for card in &deck {
                assert!(
                    defs.contains_key(&card.def_id),
                    "race {} has unknown card {:?}",
                    race,
                    card.def_id
                );
            }
        }
    }

    #[test]
    fn all_cards_start_not_upgraded() {
        let deck = fighter_starter_deck("two_handed", "human", "soldier");
        assert!(deck.iter().all(|c| !c.upgraded));
    }
}
