//! Character Creation (RunStart) — multi-step wizard.
//!
//! Steps: Race → Background → Class → Sub-class → Deck Review + Embark.
//! Creates a `GauntletRunner` on Embark and transitions to Combat.

use bevy::prelude::*;
use rand::seq::SliceRandom;

use decker_engine::class::Class;
use decker_engine::subclass::{
    fighter_subclasses, SubclassDef,
};
use decker_content::cards::{race_card_id, background_card_id};
use decker_content::starter_decks::fighter_starter_deck;
use decker_gauntlet::GauntletRunner;

use crate::state::GameState;
use crate::theme;
use crate::tooltip::TooltipContent;

pub struct RunStartPlugin;

impl Plugin for RunStartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::RunStart), setup_run_start)
            .add_systems(
                Update,
                (
                    rebuild_step_content,
                    handle_navigation,
                    handle_option_click,
                )
                    .run_if(in_state(GameState::RunStart)),
            )
            .add_systems(OnExit(GameState::RunStart), cleanup_run_start);
    }
}

// ── Data ────────────────────────────────────────────────────────────────

const RACES: &[(&str, &str)] = &[
    ("human", "Adaptable and resourceful. Jack of all trades."),
    ("high_elf", "Ancient magical heritage. Graceful and precise."),
    ("dark_elf", "Shadow-touched. Power through sacrifice."),
    ("dwarf", "Stout and resilient. Immovable in defense."),
    ("gnome", "Clever and inventive. Tricks over brute force."),
    ("halfling", "Quick and evasive. Hard to pin down."),
    ("orc", "Raw fury. Devastating charges."),
    ("goblin", "Scrappy and cunning. Death by a thousand cuts."),
    ("dragonkin", "Draconic blood. Breathe destruction."),
    ("pantheran", "Feline agility. Strike and reposition."),
];

const BACKGROUNDS: &[(&str, &str)] = &[
    ("soldier", "Military training. Disciplined strikes."),
    ("scholar", "Studied analysis. Knowledge is power."),
    ("acolyte", "Divine touched. Minor blessings."),
    ("urchin", "Street-smart. Dirty tricks."),
    ("noble", "Born to lead. Commanding presence."),
    ("outlander", "Wilderness survivor. Instinct over intellect."),
    ("artisan", "Creative problem-solver. Improvised tools."),
    ("sailor", "Sea-hardened. Steady under pressure."),
    ("criminal", "Strike from shadows. Exploit openings."),
    ("hermit", "Inward focus. Calm mind, sharp blade."),
    ("merchant", "Resourceful trader. Always prepared."),
    ("entertainer", "Captivate and dazzle. Misdirection."),
];

const CLASSES: &[(Class, &str)] = &[
    (Class::Fighter, "50 HP  |  Energy: 3/turn"),
];

fn all_subclasses() -> Vec<SubclassDef> {
    fighter_subclasses()
}

fn subclasses_for_class(class: Class) -> Vec<SubclassDef> {
    match class {
        Class::Fighter => fighter_subclasses(),
        _ => vec![],
    }
}

fn title_case(id: &str) -> String {
    id.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ── Resources ───────────────────────────────────────────────────────────

#[derive(Resource)]
struct WizardState {
    step: usize,
    last_rendered: Option<usize>,
    race_index: usize,
    /// Index into `available_backgrounds` (not BACKGROUNDS).
    background_index: usize,
    /// 3 randomly chosen indices into BACKGROUNDS for this run.
    available_backgrounds: [usize; 3],
    class_index: usize,
    subclass_index: usize,
}

impl WizardState {
    fn new(defaults: &crate::CliDefaults) -> Self {
        // Find race index from CLI default, or 0 if not specified.
        let race_index = defaults.race.as_ref()
            .and_then(|r| RACES.iter().position(|(id, _)| *id == r.as_str()))
            .unwrap_or(0);

        // Pick 3 random unique background indices.
        // If a default is specified, ensure it's included and selected.
        let mut indices: Vec<usize> = (0..BACKGROUNDS.len()).collect();
        indices.shuffle(&mut rand::rng());

        let (available_backgrounds, background_index) = if let Some(ref bg) = defaults.background {
            let bg_idx = BACKGROUNDS.iter()
                .position(|(id, _)| *id == bg.as_str())
                .unwrap_or(0);
            // Put the default first, fill remaining slots with random others.
            let others: Vec<usize> = indices.into_iter()
                .filter(|i| *i != bg_idx)
                .take(2)
                .collect();
            ([bg_idx, others[0], others[1]], 0)
        } else {
            ([indices[0], indices[1], indices[2]], 0)
        };

        Self {
            step: 0,
            last_rendered: Some(0),
            race_index,
            background_index,
            available_backgrounds,
            class_index: 0,
            subclass_index: 0,
        }
    }
}

/// The active GauntletRunner, inserted as a resource on Embark.
#[derive(Resource)]
pub struct RunData {
    pub runner: GauntletRunner,
}

// ── Components ──────────────────────────────────────────────────────────

#[derive(Component)] struct RunStartRoot;
#[derive(Component)] struct StepContent;
#[derive(Component)] struct StepTitle;
#[derive(Component)] struct BackButton;
#[derive(Component)] struct ContinueButton;
#[derive(Component)] struct ContinueButtonText;
#[derive(Component)] struct OptionButton(usize);

// ── Step titles ─────────────────────────────────────────────────────────

const STEP_TITLES: [&str; 5] = [
    "Step 1: Race",
    "Step 2: Background",
    "Step 3: Class",
    "Step 4: Sub-class",
    "Step 5: Deck Review",
];

const LAST_STEP: usize = 4;

// ── Setup ───────────────────────────────────────────────────────────────

fn setup_run_start(mut commands: Commands, defaults: Res<crate::CliDefaults>) {
    commands.insert_resource(WizardState::new(&defaults));

    commands
        .spawn((
            RunStartRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(theme::BACKGROUND),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("CHARACTER CREATION"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(theme::GOLD),
            ));

            root.spawn((
                StepTitle,
                Text::new(STEP_TITLES[0]),
                TextFont { font_size: 20.0, ..default() },
                TextColor(theme::PARCHMENT),
            ));

            root.spawn((
                StepContent,
                Node {
                    width: Val::Percent(80.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BackgroundColor(theme::PANEL),
                BorderColor::all(theme::BORDER_GOLD),
            ))
            .with_children(|content| {
                build_step_race(content, 0);
            });

            // Navigation buttons (pinned at bottom)
            root.spawn(Node {
                width: Val::Percent(80.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|nav| {
                nav.spawn((
                    BackButton,
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
                    Text::new("Back"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(theme::GOLD),
                ));

                nav.spawn((
                    ContinueButton,
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
                    ContinueButtonText,
                    Text::new("Continue"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(theme::GOLD),
                ));
            });
        });
}

// ── Rebuild step content ────────────────────────────────────────────────

fn rebuild_step_content(
    mut commands: Commands,
    mut wiz: ResMut<WizardState>,
    content_query: Query<Entity, With<StepContent>>,
    mut title_query: Query<&mut Text, (With<StepTitle>, Without<ContinueButtonText>)>,
    mut continue_text_query: Query<&mut Text, (With<ContinueButtonText>, Without<StepTitle>)>,
) {
    if wiz.last_rendered == Some(wiz.step) {
        return;
    }
    wiz.last_rendered = Some(wiz.step);
    let step = wiz.step;

    if let Ok(mut title) = title_query.single_mut() {
        **title = STEP_TITLES[step].to_string();
    }
    if let Ok(mut text) = continue_text_query.single_mut() {
        **text = if step == LAST_STEP { "Embark".to_string() } else { "Continue".to_string() };
    }

    let Ok(content_entity) = content_query.single() else { return };
    commands.entity(content_entity).despawn_children();

    let race_idx = wiz.race_index;
    let bg_idx = wiz.background_index;
    let avail_bgs = wiz.available_backgrounds;
    let class_idx = wiz.class_index;
    let sc_idx = wiz.subclass_index;

    commands.entity(content_entity).with_children(move |parent| {
        match step {
            0 => build_step_race(parent, race_idx),
            1 => build_step_background(parent, bg_idx, &avail_bgs),
            2 => build_step_class(parent, class_idx),
            3 => build_step_subclass(parent, class_idx, sc_idx),
            4 => build_step_deck_review(parent, race_idx, avail_bgs[bg_idx], class_idx, sc_idx),
            _ => {}
        }
    });
}

// ── Step builders ───────────────────────────────────────────────────────

fn spawn_option_grid(
    parent: &mut ChildSpawnerCommands,
    options: &[(&str, &str)],
    selected: usize,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(10.0),
        row_gap: Val::Px(10.0),
        ..default()
    })
    .with_children(|grid| {
        for (i, (name, desc)) in options.iter().enumerate() {
            let is_selected = i == selected;
            let border_color = if is_selected { theme::GOLD } else { theme::BORDER_SLATE };
            let bg = if is_selected { theme::PANEL_HOVER } else { theme::PANEL };

            grid.spawn((
                OptionButton(i),
                Button,
                Node {
                    width: Val::Px(180.0),
                    min_height: Val::Px(70.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border_color),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(title_case(name)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(if is_selected { theme::GOLD } else { theme::PARCHMENT }),
                ));
                card.spawn((
                    Text::new(*desc),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(theme::SLATE),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            });
        }
    });
}

fn build_step_race(parent: &mut ChildSpawnerCommands, selected: usize) {
    let content = decker_content::load_content();
    let config = decker_card_renderer::CardDisplayConfig::reward();

    // Top: option grid
    spawn_option_grid(parent, RACES, selected);

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    // Bottom: rendered card preview for current selection
    let (race_id, _) = RACES[selected];
    let card_id = race_card_id(race_id);
    if let Some(card_def) = content.card_defs.get(card_id) {
        parent.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            decker_card_renderer::spawn_card(row, card_def, &config);
        });
    }
}

fn build_step_background(parent: &mut ChildSpawnerCommands, selected: usize, available: &[usize; 3]) {
    let content = decker_content::load_content();
    let config = decker_card_renderer::CardDisplayConfig::reward();

    // Build the 3 available options
    let options: Vec<(&str, &str)> = available.iter()
        .map(|&idx| BACKGROUNDS[idx])
        .collect();

    spawn_option_grid(parent, &options, selected);

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    // Card preview for current selection
    let bg_global_idx = available[selected];
    let (bg_id, _) = BACKGROUNDS[bg_global_idx];
    let card_id = background_card_id(bg_id);
    if let Some(card_def) = content.card_defs.get(card_id) {
        parent.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            decker_card_renderer::spawn_card(row, card_def, &config);
        });
    }
}

/// The class-specific starter card ID for each class.
fn class_card_id(class: Class) -> &'static str {
    match class {
        Class::Fighter => "second_wind",
        _ => "second_wind",
    }
}

fn build_step_class(parent: &mut ChildSpawnerCommands, selected: usize) {
    let class_options: Vec<(&str, &str)> = CLASSES
        .iter()
        .map(|(c, stats)| {
            let name = match c {
                Class::Fighter => "fighter",
                _ => "fighter",
            };
            (name, *stats)
        })
        .collect();

    let (class, _) = CLASSES[selected];

    // Show class description
    parent.spawn((
        Text::new(class.description()),
        TextFont { font_size: 16.0, ..default() },
        TextColor(theme::PARCHMENT),
        TextLayout::new_with_justify(Justify::Center),
    ));

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    let options: Vec<(&str, &str)> = class_options;
    spawn_option_grid(parent, &options, selected);

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    // Show class card preview
    let content = decker_content::load_content();
    let config = decker_card_renderer::CardDisplayConfig::reward();
    let card_id = class_card_id(class);
    if let Some(card_def) = content.card_defs.get(card_id) {
        parent.spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            decker_card_renderer::spawn_card(row, card_def, &config);
        });
    }
}

fn build_step_subclass(parent: &mut ChildSpawnerCommands, class_idx: usize, selected: usize) {
    let (class, _) = CLASSES[class_idx];
    let subs = subclasses_for_class(class);

    // Prompt
    parent.spawn((
        Text::new(class.subclass_prompt()),
        TextFont { font_size: 18.0, ..default() },
        TextColor(theme::PARCHMENT),
    ));

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    // Subclass options
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        justify_content: JustifyContent::Center,
        column_gap: Val::Px(12.0),
        row_gap: Val::Px(12.0),
        ..default()
    })
    .with_children(|grid| {
        for (i, sc) in subs.iter().enumerate() {
            let is_selected = i == selected;
            let border_color = if is_selected { theme::GOLD } else { theme::BORDER_SLATE };
            let bg = if is_selected { theme::PANEL_HOVER } else { theme::PANEL };

            grid.spawn((
                OptionButton(i),
                Button,
                Node {
                    width: Val::Px(220.0),
                    min_height: Val::Px(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border_color),
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(&sc.name),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(if is_selected { theme::GOLD } else { theme::PARCHMENT }),
                ));
                card.spawn((
                    Text::new(&sc.description),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(theme::SLATE),
                    TextLayout::new_with_justify(Justify::Center),
                ));
                card.spawn((
                    Text::new(format!("\"{}\"", sc.flavor)),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(theme::SLATE),
                ));
            });
        }
    });

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    parent.spawn((
        Text::new("Sub-class cards are unlocked as rewards during the gauntlet."),
        TextFont { font_size: 12.0, ..default() },
        TextColor(theme::SLATE),
    ));
}

/// Determine the origin label for a card in the starter deck.
fn card_origin(card_id: &str, race_id: &str, bg_id: &str, class: Class) -> String {
    let race_card = race_card_id(race_id);
    let bg_card = background_card_id(bg_id);

    if card_id == race_card {
        title_case(race_id)
    } else if card_id == bg_card {
        title_case(bg_id)
    } else if card_id == "strike" || card_id == "defend" {
        "Core".into()
    } else {
        class.name().into()
    }
}

fn build_step_deck_review(
    parent: &mut ChildSpawnerCommands,
    race_idx: usize,
    bg_idx: usize,
    class_idx: usize,
    sc_idx: usize,
) {
    let (race_id, _) = RACES[race_idx];
    let (bg_id, _) = BACKGROUNDS[bg_idx];
    let (class, _) = CLASSES[class_idx];
    let subs = subclasses_for_class(class);
    let sc = &subs[sc_idx];

    let deck = fighter_starter_deck(&sc.id, race_id, bg_id);

    // Header
    parent.spawn((
        Text::new(format!(
            "{} - {}  |  {} {}  |  GAUNTLET",
            class.name(), sc.name, title_case(race_id), title_case(bg_id)
        )),
        TextFont { font_size: 18.0, ..default() },
        TextColor(theme::GOLD),
    ));

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    // Card grid (16 cards = 2 rows of 8)
    let content = decker_content::load_content();
    let config = decker_card_renderer::CardDisplayConfig::mini();
    let cards_per_row = 8;

    for row_start in (0..deck.len()).step_by(cards_per_row) {
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|row| {
            for i in row_start..(row_start + cards_per_row).min(deck.len()) {
                let card = &deck[i];
                if let Some(card_def) = content.card_defs.get(&card.def_id) {
                    let origin = card_origin(&card.def_id, race_id, bg_id, class);
                    let tooltip = format!("{}\nSource: {}", card_def.name, origin);
                    let entity = decker_card_renderer::spawn_card(row, card_def, &config);
                    row.commands().entity(entity).insert((
                        Button,
                        TooltipContent(tooltip),
                    ));
                }
            }
        });
    }

    parent.spawn(Node { height: Val::Px(8.0), ..default() });

    parent.spawn((
        Text::new("Click Embark to begin your gauntlet run!"),
        TextFont { font_size: 14.0, ..default() },
        TextColor(theme::TEAL),
    ));
}

// ── Systems ─────────────────────────────────────────────────────────────

fn handle_option_click(
    mut wiz: ResMut<WizardState>,
    query: Query<(&Interaction, &OptionButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, option) in &query {
        if *interaction != Interaction::Pressed { continue; }

        let changed = match wiz.step {
            0 => { wiz.race_index = option.0; true }
            1 => { wiz.background_index = option.0; true }
            2 => {
                if wiz.class_index != option.0 {
                    wiz.class_index = option.0;
                    wiz.subclass_index = 0; // reset subclass when class changes
                    true
                } else { false }
            }
            3 => { wiz.subclass_index = option.0; true }
            _ => false,
        };

        if changed {
            wiz.last_rendered = None; // force rebuild
        }
    }
}

fn handle_navigation(
    mut commands: Commands,
    mut wiz: ResMut<WizardState>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Back
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            if wiz.step == 0 {
                next_state.set(GameState::MainMenu);
            } else {
                wiz.step -= 1;
                wiz.last_rendered = None;
            }
        }
    }

    // Continue / Embark
    for interaction in &continue_query {
        if *interaction == Interaction::Pressed {
            if wiz.step < LAST_STEP {
                wiz.step += 1;
                wiz.last_rendered = None;
            } else {
                // Embark
                let (race_id, _) = RACES[wiz.race_index];
                let bg_global_idx = wiz.available_backgrounds[wiz.background_index];
                let (bg_id, _) = BACKGROUNDS[bg_global_idx];
                let (class, _) = CLASSES[wiz.class_index];
                let subs = subclasses_for_class(class);
                let sc = &subs[wiz.subclass_index];

                let seed = rand::random::<u64>();
                let runner = GauntletRunner::new_full(
                    &sc.id, seed, None, race_id, bg_id,
                );

                commands.insert_resource(RunData { runner });
                next_state.set(GameState::Combat);
            }
        }
    }
}

// ── Cleanup ─────────────────────────────────────────────────────────────

fn cleanup_run_start(
    mut commands: Commands,
    query: Query<Entity, With<RunStartRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<WizardState>();
}
