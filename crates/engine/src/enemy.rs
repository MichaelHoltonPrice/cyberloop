use serde::{Deserialize, Serialize};

use crate::ids::EnemyId;
use crate::status::{StatusMap, StatusType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    Attack(i32),
    Defend(i32),
    Buff(StatusType, i32),
    Debuff(StatusType, i32),
    AttackDefend(i32, i32),
    BuffAllies(StatusType, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDef {
    pub id: EnemyId,
    pub name: String,
    pub max_hp: i32,
    pub intent_pattern: Vec<IntentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyState {
    pub def_id: EnemyId,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub status_effects: StatusMap,
    pub intent_index: usize,
    pub current_intent: Option<IntentType>,
}

impl EnemyState {
    pub fn from_def(def: &EnemyDef) -> Self {
        let current_intent = def.intent_pattern.first().cloned();
        Self {
            def_id: def.id.clone(),
            hp: def.max_hp,
            max_hp: def.max_hp,
            block: 0,
            status_effects: StatusMap::new(),
            intent_index: 0,
            current_intent,
        }
    }

    pub fn advance_intent(&mut self, def: &EnemyDef) {
        if def.intent_pattern.is_empty() {
            return;
        }
        self.intent_index = (self.intent_index + 1) % def.intent_pattern.len();
        self.current_intent = Some(def.intent_pattern[self.intent_index].clone());
    }

    /// Apply damage after block absorption. Returns actual HP lost.
    pub fn take_damage(&mut self, amount: i32) -> i32 {
        let after_block = (amount - self.block).max(0);
        self.block = (self.block - amount).max(0);
        self.hp -= after_block;
        after_block
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn reset_block(&mut self) {
        self.block = 0;
    }
}

// ---------------------------------------------------------------------------
// Act scaling
// ---------------------------------------------------------------------------

/// HP multiplier by act for regular enemies (base stats are act 1).
pub fn act_hp_multiplier(act: u32) -> f64 {
    match act {
        1 => 1.0,
        2 => 1.15,
        _ => 1.3,
    }
}

/// Damage multiplier by act for regular enemies (scales Attack/Defend values).
pub fn act_damage_multiplier(act: u32) -> f64 {
    match act {
        1 => 1.0,
        2 => 1.1,
        _ => 1.2,
    }
}

/// Boss HP multiplier by act (total, not layered on top of act scaling).
pub fn boss_hp_multiplier(act: u32) -> f64 {
    match act {
        1 => 2.0,
        2 => 3.2,
        _ => 3.2,
    }
}

/// Boss damage multiplier by act (total, not layered on top of act scaling).
pub fn boss_damage_multiplier(act: u32) -> f64 {
    match act {
        1 => 1.6,
        2 => 1.7,
        _ => 1.7,
    }
}

/// Scale a single intent's numeric values by the damage multiplier.
/// Buff/Debuff stack counts are left unchanged.
fn scale_intent(intent: &IntentType, dmg_mult: f64) -> IntentType {
    match intent {
        IntentType::Attack(v) => IntentType::Attack((*v as f64 * dmg_mult).round() as i32),
        IntentType::Defend(v) => IntentType::Defend((*v as f64 * dmg_mult).round() as i32),
        IntentType::AttackDefend(a, d) => IntentType::AttackDefend(
            (*a as f64 * dmg_mult).round() as i32,
            (*d as f64 * dmg_mult).round() as i32,
        ),
        IntentType::Buff(st, n) => IntentType::Buff(*st, *n),
        IntentType::Debuff(st, n) => IntentType::Debuff(*st, *n),
        IntentType::BuffAllies(st, n) => IntentType::BuffAllies(*st, *n),
    }
}

/// Return a new EnemyDef scaled with arbitrary HP and damage multipliers.
pub fn scale_enemy_def_custom(def: &EnemyDef, hp_mult: f64, dmg_mult: f64) -> EnemyDef {
    EnemyDef {
        id: def.id.clone(),
        name: def.name.clone(),
        max_hp: (def.max_hp as f64 * hp_mult).round() as i32,
        intent_pattern: def
            .intent_pattern
            .iter()
            .map(|i| scale_intent(i, dmg_mult))
            .collect(),
    }
}

/// Return a new EnemyDef with HP and intent pattern scaled for the given act.
pub fn scale_enemy_def(def: &EnemyDef, act: u32) -> EnemyDef {
    scale_enemy_def_custom(def, act_hp_multiplier(act), act_damage_multiplier(act))
}

/// Return a new EnemyDef scaled with boss-specific multipliers (total, not layered).
pub fn scale_boss_def(def: &EnemyDef, act: u32) -> EnemyDef {
    scale_enemy_def_custom(def, boss_hp_multiplier(act), boss_damage_multiplier(act))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_def() -> EnemyDef {
        EnemyDef {
            id: "slime".to_string(),
            name: "Slime".to_string(),
            max_hp: 30,
            intent_pattern: vec![
                IntentType::Attack(6),
                IntentType::Defend(5),
                IntentType::Attack(10),
            ],
        }
    }

    #[test]
    fn intent_cycling() {
        let def = test_def();
        let mut enemy = EnemyState::from_def(&def);
        assert!(matches!(enemy.current_intent, Some(IntentType::Attack(6))));

        enemy.advance_intent(&def);
        assert!(matches!(enemy.current_intent, Some(IntentType::Defend(5))));

        enemy.advance_intent(&def);
        assert!(matches!(enemy.current_intent, Some(IntentType::Attack(10))));

        enemy.advance_intent(&def);
        assert!(matches!(enemy.current_intent, Some(IntentType::Attack(6))));
    }

    #[test]
    fn take_damage_with_block() {
        let def = test_def();
        let mut enemy = EnemyState::from_def(&def);
        enemy.block = 5;
        let lost = enemy.take_damage(8);
        assert_eq!(lost, 3);
        assert_eq!(enemy.hp, 27);
        assert_eq!(enemy.block, 0);
    }

    #[test]
    fn is_dead_check() {
        let def = test_def();
        let mut enemy = EnemyState::from_def(&def);
        assert!(!enemy.is_dead());
        enemy.hp = 0;
        assert!(enemy.is_dead());
    }

    #[test]
    fn string_id_roundtrip() {
        let def = test_def();
        assert_eq!(def.id, "slime");
        let enemy = EnemyState::from_def(&def);
        assert_eq!(enemy.def_id, "slime");
    }
}
