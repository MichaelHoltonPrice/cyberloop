//! Python bindings for the Decker gauntlet engine.
//!
//! Provides two environment interfaces for Python:
//!
//! - **`GauntletEnv`** — single-environment gym-style wrapper for evaluation
//!   and non-vectorized use.
//! - **`VecGauntletEnv`** — vectorized environment that holds N game runners
//!   and steps them all in a single PyO3 call.  Returns pre-shaped numpy
//!   float32 arrays to avoid Python list→tensor conversion overhead.
//!   Used by `train_ppo.py` and `train_phased.py` for efficient rollout collection.
//!
//! # Feature encoding
//!
//! The `features` module converts game state into normalized feature vectors:
//! - Global features (210-dim): player vitals, statuses, ability scores,
//!   card pile histograms (draw/hand/discard/exhaust × 46 card vocabulary).
//! - Per-card features (7-dim): cost, tags, playability.
//! - Per-enemy features (24-dim): HP, intent type/magnitude/status, statuses.
//! - Per-action features (21-dim): action type, card features, target info.
//!
//! # Example usage
//!
//! ```python
//! import decker_pyenv as decker
//!
//! # Single environment (for evaluation):
//! env = decker.GauntletEnv("two_handed", seed=42)
//! obs = env.observe()
//! actions = env.legal_actions()
//! obs, reward, done = env.step(actions[0])
//!
//! # Vectorized environments (for training):
//! vec = decker.VecGauntletEnv("defense", [42, 43, 44, 45])
//! g, h, e, a, m, nh, ne, na = vec.get_obs_all()  # numpy arrays
//! g, h, e, a, m, nh, ne, na, rewards, dones, fights, phase_ids, fight_hp, fight_max_hp = vec.step_all([0, 0, 0, 0])
//! ```

mod features;

use std::collections::HashMap;

use numpy::{IntoPyArray, PyArray1, PyArray2, PyArrayMethods};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use decker_engine::rng::GameRng;
use decker_gauntlet::{GauntletEvent, GauntletPhase, GauntletRunner};

use features::{
    all_enemy_features, FeatureConfig, ACTION_FEAT_DIM, CARD_FEAT_DIM, ENEMY_FEAT_DIM, GLOBAL_DIM,
    MAX_ACTIONS, MAX_ENEMIES, MAX_HAND, VOCAB_SIZE,
};

type FeatureSnapshot = (
    Vec<f64>,
    Vec<Vec<f64>>,
    Vec<Vec<f64>>,
    Vec<Vec<f64>>,
    usize,
);

fn deck_ids(runner: &GauntletRunner) -> Vec<String> {
    runner
        .play_deck
        .iter()
        .map(|card| card.def_id.clone())
        .collect()
}

fn multiset_cards_changed(before: &[String], after: &[String]) -> usize {
    let mut before_counts = HashMap::new();
    let mut after_counts = HashMap::new();
    for card in before {
        *before_counts.entry(card.clone()).or_insert(0usize) += 1;
    }
    for card in after {
        *after_counts.entry(card.clone()).or_insert(0usize) += 1;
    }

    let all_cards: std::collections::HashSet<String> = before_counts
        .keys()
        .chain(after_counts.keys())
        .cloned()
        .collect();
    all_cards
        .iter()
        .map(|card| {
            before_counts
                .get(card)
                .copied()
                .unwrap_or(0)
                .abs_diff(after_counts.get(card).copied().unwrap_or(0))
        })
        .sum::<usize>()
        / 2
}

fn build_feature_snapshot(runner: &GauntletRunner, fc: &FeatureConfig) -> FeatureSnapshot {
    let obs = runner.observe();
    let gf = fc.global_features(&obs);
    let hf = fc.hand_features(&obs);
    let ef = all_enemy_features(&obs);
    let af = fc.all_action_features(runner);
    let n_legal = af.len();
    (gf, hf, ef, af, n_legal)
}

/// Build a FeatureConfig for a given subclass, optional synergy filter, race, and background.
fn make_feature_config(subclass: &str, synergy_filter: Option<&str>, race: &str, background: &str) -> FeatureConfig {
    let active = decker_content::cards::active_card_ids_for_run(subclass, synergy_filter, race, background);
    FeatureConfig::new(&active)
}

// ── GauntletEnv ──────────────────────────────────────────────────────────

/// Python-facing gauntlet environment with a gym-style step interface.
#[pyclass]
struct GauntletEnv {
    runner: GauntletRunner,
    last_fight_outcome: Option<(bool, i32, i32)>,
    fc: FeatureConfig,
    synergy_filter: Option<String>,
}

fn extract_last_fight_outcome(events: &[GauntletEvent]) -> Option<(bool, i32, i32)> {
    for event in events.iter().rev() {
        match event {
            GauntletEvent::FightWon {
                player_hp,
                player_max_hp,
                ..
            } => return Some((true, *player_hp, *player_max_hp)),
            GauntletEvent::FightLost {
                player_hp,
                player_max_hp,
                ..
            } => return Some((false, *player_hp, *player_max_hp)),
            _ => {}
        }
    }
    None
}

#[pymethods]
impl GauntletEnv {
    /// Create a new gauntlet environment.
    ///
    /// Args:
    ///     subclass: Fighter sub-class id ("two_handed", "defense", "dueling")
    ///     seed: RNG seed for reproducibility
    ///     race: Player race ("human", "high_elf", "dark_elf", etc.)
    ///     synergy_filter: Optional synergy group filter ("marked", "tempo")
    #[new]
    #[pyo3(signature = (subclass, seed, race, background="soldier", synergy_filter=None))]
    fn new(subclass: &str, seed: u64, race: &str, background: &str, synergy_filter: Option<&str>) -> Self {
        let fc = make_feature_config(subclass, synergy_filter, race, background);
        Self {
            runner: GauntletRunner::new_full(subclass, seed, synergy_filter, race, background),
            last_fight_outcome: None,
            fc,
            synergy_filter: synergy_filter.map(|s| s.to_string()),
        }
    }

    /// Reconstruct an environment from a save file JSON string.
    #[staticmethod]
    fn from_save(json: &str) -> PyResult<Self> {
        let runner = GauntletRunner::from_save(json)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;
        let subclass = runner.subclass_id.clone();
        let synergy = runner.synergy_filter.clone();
        let race = runner.race.clone();
        let background = runner.background.clone();
        let fc = make_feature_config(&subclass, synergy.as_deref(), &race, &background);
        Ok(Self {
            runner,
            last_fight_outcome: None,
            fc,
            synergy_filter: synergy,
        })
    }

    /// Serialize the current state to a save file JSON string.
    fn save(&self) -> PyResult<String> {
        serde_json::to_string_pretty(&self.runner)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Returns a JSON string of the current observation.
    fn observe(&self) -> PyResult<String> {
        let obs = self.runner.observe();
        serde_json::to_string(&obs)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Returns a list of legal action indices.
    fn legal_actions(&self) -> Vec<usize> {
        let actions = self.runner.legal_actions();
        (0..actions.len()).collect()
    }

    /// Returns a list of human-readable action descriptions.
    fn legal_action_labels(&self) -> Vec<String> {
        self.runner
            .legal_actions()
            .iter()
            .map(|a| format!("{:?}", a))
            .collect()
    }

    /// Execute an action by index. Returns (observation_json, reward, done).
    fn step(&mut self, action_index: usize) -> PyResult<(String, f64, bool)> {
        let actions = self.runner.legal_actions();
        if action_index >= actions.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "action_index {} out of range (0..{})",
                action_index,
                actions.len()
            )));
        }

        let prev_fights = self.runner.fights_won;
        self.last_fight_outcome = None;

        let action = actions[action_index].clone();
        let events = self.runner
            .apply(action)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        self.last_fight_outcome = extract_last_fight_outcome(&events);

        let obs = self.runner.observe();
        let obs_json = serde_json::to_string(&obs)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        let done = self.runner.is_game_over();
        let reward = if self.runner.fights_won > prev_fights {
            1.0
        } else {
            0.0
        };

        Ok((obs_json, reward, done))
    }

    /// Number of fights won so far.
    #[getter]
    fn fights_won(&self) -> u32 {
        self.runner.fights_won
    }

    /// Current player HP.
    #[getter]
    fn player_hp(&self) -> i32 {
        self.runner.player.hp
    }

    /// Whether the run is over.
    #[getter]
    fn is_done(&self) -> bool {
        self.runner.is_game_over()
    }

    /// Reset the environment with a new seed.
    #[pyo3(signature = (subclass, seed, race, background="soldier", synergy_filter=None))]
    fn reset(&mut self, subclass: &str, seed: u64, race: &str, background: &str, synergy_filter: Option<&str>) -> PyResult<String> {
        // Use provided filter, or fall back to the one from construction.
        let filter = synergy_filter.or(self.synergy_filter.as_deref());
        self.runner = GauntletRunner::new_full(subclass, seed, filter, race, background);
        self.fc = make_feature_config(subclass, filter, race, background);
        self.last_fight_outcome = None;
        self.observe()
    }

    // ── Structured feature methods for RL ─────────────────────────────

    /// Returns normalized global features as a flat list of floats.
    fn get_global_features(&self) -> Vec<f64> {
        let obs = self.runner.observe();
        self.fc.global_features(&obs)
    }

    /// Returns per-card features for the current hand.
    /// Each inner list has card_feat_dim elements.
    fn get_hand_features(&self) -> Vec<Vec<f64>> {
        let obs = self.runner.observe();
        self.fc.hand_features(&obs)
    }

    /// Returns per-enemy features for alive enemies.
    /// Each inner list has ENEMY_FEAT_DIM elements.
    fn get_enemy_features(&self) -> Vec<Vec<f64>> {
        let obs = self.runner.observe();
        all_enemy_features(&obs)
    }

    /// Returns per-action feature vectors for all legal actions.
    /// Each inner list has action_feat_dim elements.
    fn get_action_features(&self) -> Vec<Vec<f64>> {
        self.fc.all_action_features(&self.runner)
    }

    /// Execute an action and return structured features instead of JSON.
    /// Returns (global_feats, hand_feats, enemy_feats, action_feats, reward, done, n_legal).
    #[allow(clippy::type_complexity)]
    fn step_features(
        &mut self,
        action_index: usize,
    ) -> PyResult<(
        Vec<f64>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        f64,
        bool,
        usize,
    )> {
        let actions = self.runner.legal_actions();
        if action_index >= actions.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "action_index {} out of range (0..{})",
                action_index,
                actions.len()
            )));
        }

        let prev_fights = self.runner.fights_won;
        self.last_fight_outcome = None;

        let action = actions[action_index].clone();
        let events = self.runner
            .apply(action)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;
        self.last_fight_outcome = extract_last_fight_outcome(&events);

        let done = self.runner.is_game_over();
        let reward = if self.runner.fights_won > prev_fights {
            1.0
        } else {
            0.0
        };

        let obs = self.runner.observe();
        let gf = self.fc.global_features(&obs);
        let hf = self.fc.hand_features(&obs);
        let ef = all_enemy_features(&obs);
        let af = self.fc.all_action_features(&self.runner);

        let n_legal = af.len();
        Ok((gf, hf, ef, af, reward, done, n_legal))
    }

    /// Outcome of the most recently completed fight, if the last step ended one.
    ///
    /// Returns (won, player_hp_at_combat_end, player_max_hp_at_combat_end).
    fn last_fight_outcome(&self) -> Option<(bool, i32, i32)> {
        self.last_fight_outcome
    }

    /// Get the default (unfiltered) feature dimensions.
    /// For filtered dimensions, use instance_dims() after constructing with synergy_filter.
    #[staticmethod]
    fn feature_dims() -> (usize, usize, usize, usize, usize) {
        (
            GLOBAL_DIM,
            CARD_FEAT_DIM,
            ENEMY_FEAT_DIM,
            ACTION_FEAT_DIM,
            VOCAB_SIZE,
        )
    }

    /// Get feature dimensions for this specific environment instance.
    /// When a synergy filter is active, these will be smaller than feature_dims().
    fn instance_dims(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.fc.global_dim,
            self.fc.card_feat_dim,
            ENEMY_FEAT_DIM,
            self.fc.action_feat_dim,
            self.fc.vocab_size,
        )
    }
}

// ── RebuildCurriculumEnv ─────────────────────────────────────────────────

/// Training-only environment that starts at a short-rest rebuild and scores
/// the next small fight horizon after the ordered rewrite completes.
#[pyclass]
struct RebuildCurriculumEnv {
    runner: GauntletRunner,
    subclass: String,
    race: String,
    background: String,
    horizon_fights: u32,
    change_bonus: f64,
    identity_penalty: f64,
    baseline_fights: u32,
    rebuild_start_deck: Vec<String>,
    done: bool,
    fc: FeatureConfig,
}

impl RebuildCurriculumEnv {
    fn setup_rebuild_scenario(&mut self, seed: u64) -> PyResult<()> {
        let mut rng = GameRng::new(seed);
        let mut scenario_seed = seed;
        for _ in 0..32 {
            let mut runner = GauntletRunner::new_full(&self.subclass, scenario_seed, None, &self.race, &self.background);
            let mut steps = 0usize;
            while !runner.is_game_over() && steps < 500_000 {
                if matches!(runner.phase, GauntletPhase::DeckRebuild { .. }) {
                    self.baseline_fights = runner.fights_won;
                    self.rebuild_start_deck = deck_ids(&runner);
                    self.done = false;
                    self.runner = runner;
                    return Ok(());
                }
                let legal = runner.legal_actions();
                if legal.is_empty() {
                    break;
                }
                let idx = rng.range(0, legal.len() as i32 - 1) as usize;
                runner.apply(legal[idx].clone()).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "failed to prepare rebuild scenario: {:?}",
                        e
                    ))
                })?;
                steps += 1;
            }
            scenario_seed = scenario_seed.wrapping_add(1);
        }
        Err(pyo3::exceptions::PyRuntimeError::new_err(
            "failed to reach a rebuild scenario within 32 seeds",
        ))
    }

    fn autoplay_horizon(&mut self) -> PyResult<f64> {
        let mut rng = GameRng::new(self.runner.fights_won as u64 + 9999);
        let target_fights = self.baseline_fights + self.horizon_fights;
        while !self.runner.is_game_over() && self.runner.fights_won < target_fights {
            let legal = self.runner.legal_actions();
            if legal.is_empty() {
                break;
            }
            let idx = rng.range(0, legal.len() as i32 - 1) as usize;
            self.runner.apply(legal[idx].clone()).map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "failed during curriculum autoplay: {:?}",
                    e
                ))
            })?;
        }
        Ok((self.runner.fights_won - self.baseline_fights) as f64)
    }
}

#[pymethods]
impl RebuildCurriculumEnv {
    #[new]
    #[pyo3(signature = (subclass, seed, race, background="soldier", horizon_fights=20, change_bonus=None, identity_penalty=None))]
    fn new(
        subclass: &str,
        seed: u64,
        race: &str,
        background: &str,
        horizon_fights: u32,
        change_bonus: Option<f64>,
        identity_penalty: Option<f64>,
    ) -> PyResult<Self> {
        let fc = make_feature_config(subclass, None, race, background);
        let mut env = Self {
            runner: GauntletRunner::new_full(subclass, seed, None, race, background),
            subclass: subclass.to_string(),
            race: race.to_string(),
            background: background.to_string(),
            horizon_fights,
            change_bonus: change_bonus.unwrap_or(0.0),
            identity_penalty: identity_penalty.unwrap_or(0.0),
            baseline_fights: 0,
            rebuild_start_deck: Vec::new(),
            done: false,
            fc,
        };
        env.setup_rebuild_scenario(seed)?;
        Ok(env)
    }

    fn observe(&self) -> PyResult<String> {
        let obs = self.runner.observe();
        serde_json::to_string(&obs)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    fn legal_action_labels(&self) -> Vec<String> {
        self.runner
            .legal_actions()
            .iter()
            .map(|a| format!("{:?}", a))
            .collect()
    }

    #[allow(clippy::type_complexity)]
    fn step_features(
        &mut self,
        action_index: usize,
    ) -> PyResult<(
        Vec<f64>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        f64,
        bool,
        usize,
    )> {
        if self.done {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "environment is done; call reset()",
            ));
        }
        let actions = self.runner.legal_actions();
        if action_index >= actions.len() {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "action_index {} out of range (0..{})",
                action_index,
                actions.len()
            )));
        }

        let action = actions[action_index].clone();
        self.runner
            .apply(action)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e)))?;

        let mut reward = 0.0;
        if matches!(self.runner.phase, GauntletPhase::DeckRebuild { .. }) {
            let (gf, hf, ef, af, n_legal) = build_feature_snapshot(&self.runner, &self.fc);
            return Ok((gf, hf, ef, af, reward, false, n_legal));
        }

        let cards_changed =
            multiset_cards_changed(&self.rebuild_start_deck, &deck_ids(&self.runner));
        if cards_changed > 0 {
            reward += self.change_bonus;
        } else {
            reward -= self.identity_penalty;
        }
        reward += self.autoplay_horizon()?;
        self.done = true;

        let (gf, hf, ef, af, n_legal) = build_feature_snapshot(&self.runner, &self.fc);
        Ok((gf, hf, ef, af, reward, true, n_legal))
    }

    fn reset(&mut self, seed: u64) -> PyResult<String> {
        self.setup_rebuild_scenario(seed)?;
        self.observe()
    }

    fn get_global_features(&self) -> Vec<f64> {
        let obs = self.runner.observe();
        self.fc.global_features(&obs)
    }

    fn get_hand_features(&self) -> Vec<Vec<f64>> {
        let obs = self.runner.observe();
        self.fc.hand_features(&obs)
    }

    fn get_enemy_features(&self) -> Vec<Vec<f64>> {
        let obs = self.runner.observe();
        all_enemy_features(&obs)
    }

    fn get_action_features(&self) -> Vec<Vec<f64>> {
        self.fc.all_action_features(&self.runner)
    }

    #[getter]
    fn fights_won(&self) -> u32 {
        self.runner.fights_won
    }

    #[getter]
    fn is_done(&self) -> bool {
        self.done
    }

    #[staticmethod]
    fn feature_dims() -> (usize, usize, usize, usize, usize) {
        (
            GLOBAL_DIM,
            CARD_FEAT_DIM,
            ENEMY_FEAT_DIM,
            ACTION_FEAT_DIM,
            VOCAB_SIZE,
        )
    }
}

// ── Reward ───────────────────────────────────────────────────────────────
// +1.0 per fight won, 0.0 otherwise. No shaping — optimizes directly for
// the metric we care about (total fights won per run).

// ── VecGauntletEnv ──────────────────────────────────────────────────────

/// Vectorized environment: holds N gauntlet runners and steps them all in one call.
///
/// Returns pre-padded numpy arrays directly, avoiding Python list intermediaries.
#[pyclass]
struct VecGauntletEnv {
    runners: Vec<GauntletRunner>,
    subclass: String,
    race: String,
    background: String,
    n_envs: usize,
    next_reset_seeds: Vec<u64>,
    fc: FeatureConfig,
    synergy_filter: Option<String>,
}

/// Raw observation buffers built in Rust, ready for numpy conversion.
struct ObsBufs {
    g: Vec<f32>,
    h: Vec<f32>,
    e: Vec<f32>,
    a: Vec<f32>,
    m: Vec<f32>,
    nh: Vec<i32>,
    ne: Vec<i32>,
    na: Vec<i32>,
}

impl VecGauntletEnv {
    fn build_obs_bufs(&self) -> ObsBufs {
        let n = self.n_envs;
        let global_dim = self.fc.global_dim;
        let card_feat_dim = self.fc.card_feat_dim;
        let action_feat_dim = self.fc.action_feat_dim;

        let mut g = vec![0.0f32; n * global_dim];
        let mut h = vec![0.0f32; n * MAX_HAND * card_feat_dim];
        let mut e = vec![0.0f32; n * MAX_ENEMIES * ENEMY_FEAT_DIM];
        let mut a = vec![0.0f32; n * MAX_ACTIONS * action_feat_dim];
        let mut m = vec![0.0f32; n * MAX_ACTIONS];
        let mut nh = vec![0i32; n];
        let mut ne = vec![0i32; n];
        let mut na = vec![0i32; n];

        for (idx, runner) in self.runners.iter().enumerate() {
            let obs = runner.observe();
            let gf = self.fc.global_features(&obs);
            let hf = self.fc.hand_features(&obs);
            let ef = all_enemy_features(&obs);
            let af = self.fc.all_action_features(runner);

            let g_off = idx * global_dim;
            for (j, &v) in gf.iter().enumerate() {
                g[g_off + j] = v as f32;
            }

            let n_hand = hf.len().min(MAX_HAND);
            let h_off = idx * MAX_HAND * card_feat_dim;
            for i in 0..n_hand {
                for (j, &v) in hf[i].iter().enumerate() {
                    h[h_off + i * card_feat_dim + j] = v as f32;
                }
            }

            let n_enemy = ef.len().min(MAX_ENEMIES);
            let e_off = idx * MAX_ENEMIES * ENEMY_FEAT_DIM;
            for i in 0..n_enemy {
                for (j, &v) in ef[i].iter().enumerate() {
                    e[e_off + i * ENEMY_FEAT_DIM + j] = v as f32;
                }
            }

            let n_act = af.len().min(MAX_ACTIONS);
            let a_off = idx * MAX_ACTIONS * action_feat_dim;
            let m_off = idx * MAX_ACTIONS;
            for i in 0..n_act {
                for (j, &v) in af[i].iter().enumerate() {
                    a[a_off + i * action_feat_dim + j] = v as f32;
                }
                m[m_off + i] = 1.0;
            }

            nh[idx] = n_hand as i32;
            ne[idx] = n_enemy as i32;
            na[idx] = n_act as i32;
        }

        ObsBufs {
            g,
            h,
            e,
            a,
            m,
            nh,
            ne,
            na,
        }
    }
}

#[pymethods]
impl VecGauntletEnv {
    #[new]
    #[pyo3(signature = (subclass, seeds, race, background="soldier", synergy_filter=None))]
    fn new(subclass: &str, seeds: Vec<u64>, race: &str, background: &str, synergy_filter: Option<&str>) -> Self {
        let n_envs = seeds.len();
        let fc = make_feature_config(subclass, synergy_filter, race, background);
        let runners = seeds
            .iter()
            .map(|&s| GauntletRunner::new_full(subclass, s, synergy_filter, race, background))
            .collect();
        let next_reset_seeds = seeds
            .iter()
            .map(|&s| {
                s.wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407)
            })
            .collect();
        Self {
            runners,
            subclass: subclass.to_string(),
            race: race.to_string(),
            background: background.to_string(),
            n_envs,
            next_reset_seeds,
            fc,
            synergy_filter: synergy_filter.map(|s| s.to_string()),
        }
    }

    /// Get observations for all environments as numpy arrays.
    ///
    /// Returns: (global, hand, enemies, action_feats, action_mask, n_hand, n_enemies, n_actions)
    ///   - global:       shape (n_envs, GLOBAL_DIM)       float32
    ///   - hand:         shape (n_envs, MAX_HAND, CARD_FEAT_DIM) float32
    ///   - enemies:      shape (n_envs, MAX_ENEMIES, ENEMY_FEAT_DIM) float32
    ///   - action_feats: shape (n_envs, MAX_ACTIONS, ACTION_FEAT_DIM) float32
    ///   - action_mask:  shape (n_envs, MAX_ACTIONS) float32
    ///   - n_hand:       shape (n_envs,) int32
    ///   - n_enemies:    shape (n_envs,) int32
    ///   - n_actions:    shape (n_envs,) int32
    #[allow(clippy::type_complexity)]
    fn get_obs_all<'py>(
        &self,
        py: Python<'py>,
    ) -> (
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray1<i32>>,
        Bound<'py, PyArray1<i32>>,
        Bound<'py, PyArray1<i32>>,
    ) {
        let bufs = self.build_obs_bufs();
        let n = self.n_envs;
        let global_dim = self.fc.global_dim;
        let card_feat_dim = self.fc.card_feat_dim;
        let action_feat_dim = self.fc.action_feat_dim;
        (
            bufs.g.into_pyarray(py).reshape([n, global_dim]).unwrap(),
            bufs.h
                .into_pyarray(py)
                .reshape([n, MAX_HAND * card_feat_dim])
                .unwrap(),
            bufs.e
                .into_pyarray(py)
                .reshape([n, MAX_ENEMIES * ENEMY_FEAT_DIM])
                .unwrap(),
            bufs.a
                .into_pyarray(py)
                .reshape([n, MAX_ACTIONS * action_feat_dim])
                .unwrap(),
            bufs.m.into_pyarray(py).reshape([n, MAX_ACTIONS]).unwrap(),
            bufs.nh.into_pyarray(py),
            bufs.ne.into_pyarray(py),
            bufs.na.into_pyarray(py),
        )
    }

    /// Step all environments with the given actions. Auto-resets done envs.
    ///
    /// Returns numpy arrays: (global, hand, enemies, action_feats, action_mask,
    ///     n_hand, n_enemies, n_actions, rewards, dones, fights_won,
    ///     phase_ids, fight_hp, fight_max_hp)
    ///
    /// The last three arrays support phased training:
    ///   - phase_ids: phase index *before* stepping (matches PHASE_NAMES order)
    ///   - fight_hp:  player HP at end of fight (0.0 if no fight ended)
    ///   - fight_max_hp: player max HP at end of fight (0.0 if no fight ended)
    fn step_all<'py>(
        &mut self,
        py: Python<'py>,
        actions: Vec<usize>,
    ) -> PyResult<Bound<'py, PyTuple>> {
        if actions.len() != self.n_envs {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "expected {} actions, got {}",
                self.n_envs,
                actions.len()
            )));
        }

        let mut rewards = Vec::with_capacity(self.n_envs);
        let mut dones = Vec::with_capacity(self.n_envs);
        let mut fights = Vec::with_capacity(self.n_envs);
        let mut phase_ids = Vec::with_capacity(self.n_envs);
        let mut fight_hp = Vec::with_capacity(self.n_envs);
        let mut fight_max_hp = Vec::with_capacity(self.n_envs);

        for (i, runner) in self.runners.iter_mut().enumerate() {
            // Record phase before stepping (matches PHASE_NAMES order in strategy.py).
            let phase_id = match &runner.phase {
                GauntletPhase::Combat => 0i32,
                GauntletPhase::Reward => 1,
                GauntletPhase::CollectionOverflow { .. } => 2,
                GauntletPhase::InnateChoice => 3,
                GauntletPhase::DeckSwap { .. } => 4,
                GauntletPhase::DeckRebuild { .. } => 5,
                GauntletPhase::GameOver => 6,
            };
            phase_ids.push(phase_id);

            let legal = runner.legal_actions();
            let act = if actions[i] < legal.len() {
                actions[i]
            } else {
                0
            };
            let prev_fights = runner.fights_won;

            let action = legal[act].clone();
            let events = runner.apply(action).map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("{:?}", e))
            })?;

            // Extract fight outcome.
            match extract_last_fight_outcome(&events) {
                Some((_won, hp, max_hp)) => {
                    fight_hp.push(hp as f32);
                    fight_max_hp.push(max_hp as f32);
                }
                None => {
                    fight_hp.push(0.0f32);
                    fight_max_hp.push(0.0f32);
                }
            }

            let done = runner.is_game_over();
            let reward = if runner.fights_won > prev_fights {
                1.0f32
            } else {
                0.0f32
            };

            rewards.push(reward);
            dones.push(if done { 1.0f32 } else { 0.0f32 });
            fights.push(runner.fights_won);

            if done {
                let new_seed = self.next_reset_seeds[i];
                *runner = GauntletRunner::new_full(
                    &self.subclass,
                    new_seed,
                    self.synergy_filter.as_deref(),
                    &self.race,
                    &self.background,
                );
                self.next_reset_seeds[i] = new_seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
            }
        }

        let bufs = self.build_obs_bufs();
        let n = self.n_envs;
        let global_dim = self.fc.global_dim;
        let card_feat_dim = self.fc.card_feat_dim;
        let action_feat_dim = self.fc.action_feat_dim;
        PyTuple::new(py, [
            bufs.g.into_pyarray(py).reshape([n, global_dim]).unwrap().into_any(),
            bufs.h
                .into_pyarray(py)
                .reshape([n, MAX_HAND * card_feat_dim])
                .unwrap()
                .into_any(),
            bufs.e
                .into_pyarray(py)
                .reshape([n, MAX_ENEMIES * ENEMY_FEAT_DIM])
                .unwrap()
                .into_any(),
            bufs.a
                .into_pyarray(py)
                .reshape([n, MAX_ACTIONS * action_feat_dim])
                .unwrap()
                .into_any(),
            bufs.m.into_pyarray(py).reshape([n, MAX_ACTIONS]).unwrap().into_any(),
            bufs.nh.into_pyarray(py).into_any(),
            bufs.ne.into_pyarray(py).into_any(),
            bufs.na.into_pyarray(py).into_any(),
            rewards.into_pyarray(py).into_any(),
            dones.into_pyarray(py).into_any(),
            fights.into_pyarray(py).into_any(),
            phase_ids.into_pyarray(py).into_any(),
            fight_hp.into_pyarray(py).into_any(),
            fight_max_hp.into_pyarray(py).into_any(),
        ])
    }

    /// Reset a single environment.
    fn reset_env(&mut self, idx: usize, seed: u64) -> PyResult<()> {
        if idx >= self.n_envs {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "index out of range",
            ));
        }
        self.runners[idx] = GauntletRunner::new_full(
            &self.subclass,
            seed,
            self.synergy_filter.as_deref(),
            &self.race,
            &self.background,
        );
        self.next_reset_seeds[idx] = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        Ok(())
    }

    /// Number of environments.
    #[getter]
    fn n_envs(&self) -> usize {
        self.n_envs
    }

    /// Default (unfiltered) feature dimensions and padding constants.
    #[staticmethod]
    fn dims() -> (usize, usize, usize, usize, usize, usize, usize, usize) {
        (
            GLOBAL_DIM,
            CARD_FEAT_DIM,
            ENEMY_FEAT_DIM,
            ACTION_FEAT_DIM,
            VOCAB_SIZE,
            MAX_HAND,
            MAX_ENEMIES,
            MAX_ACTIONS,
        )
    }

    /// Feature dimensions for this specific instance (respects synergy filter).
    fn instance_dims(&self) -> (usize, usize, usize, usize, usize, usize, usize, usize) {
        (
            self.fc.global_dim,
            self.fc.card_feat_dim,
            ENEMY_FEAT_DIM,
            self.fc.action_feat_dim,
            self.fc.vocab_size,
            MAX_HAND,
            MAX_ENEMIES,
            MAX_ACTIONS,
        )
    }
}

// ── Batch simulation helper ──────────────────────────────────────────────

/// Run a batch of gauntlet simulations with a random agent.
///
/// Args:
///     subclass: Fighter sub-class id
///     seeds: list of RNG seeds
///
/// Returns:
///     List of dicts with keys: seed, fights_won, final_hp, final_deck_size, total_steps
#[pyfunction]
fn batch_simulate(
    py: Python<'_>,
    subclass: &str,
    race: &str,
    background: &str,
    _agent: &str,
    seeds: Vec<u64>,
) -> PyResult<Vec<Py<PyDict>>> {
    let mut results = Vec::with_capacity(seeds.len());

    for &seed in &seeds {
        let mut rng = GameRng::new(seed.wrapping_add(1));
        let mut runner = GauntletRunner::new_full(subclass, seed, None, race, background);
        let max_steps = 500_000usize;
        let mut steps = 0usize;

        while !runner.is_game_over() && steps < max_steps {
            let legal = runner.legal_actions();
            if legal.is_empty() {
                break;
            }
            let idx = rng.range(0, legal.len() as i32 - 1) as usize;
            let _ = runner.apply(legal[idx].clone());
            steps += 1;
        }

        let dict = PyDict::new(py);
        dict.set_item("seed", seed)?;
        dict.set_item("fights_won", runner.fights_won)?;
        dict.set_item("final_hp", runner.player.hp)?;
        dict.set_item("final_deck_size", runner.play_deck.len())?;
        dict.set_item("total_steps", steps)?;
        results.push(dict.into());
    }

    Ok(results)
}

// ── Engine version ───────────────────────────────────────────────────────

/// Returns the current engine version as a JSON string.
///
/// The version includes:
///   - semver: workspace version from Cargo.toml
///   - git_commit: full commit hash at build time
///   - content_hash: SHA-256 of all gameplay-relevant definitions
///
/// Two versions are compatible iff semver AND content_hash match.
#[pyfunction]
fn engine_version() -> PyResult<String> {
    let version = decker_content::engine_version();
    serde_json::to_string(&version)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

// ── Synergy group helpers ────────────────────────────────────────────────

/// Return the subclass that owns a synergy group, or None if unknown.
#[pyfunction]
fn subclass_for_synergy_group(group: &str) -> Option<String> {
    decker_content::cards::subclass_for_synergy_group(group).map(|s| s.to_string())
}

// ── Python module ────────────────────────────────────────────────────────

#[pymodule]
fn decker_pyenv(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GauntletEnv>()?;
    m.add_class::<RebuildCurriculumEnv>()?;
    m.add_class::<VecGauntletEnv>()?;
    m.add_function(wrap_pyfunction!(batch_simulate, m)?)?;
    m.add_function(wrap_pyfunction!(engine_version, m)?)?;
    m.add_function(wrap_pyfunction!(subclass_for_synergy_group, m)?)?;
    Ok(())
}
