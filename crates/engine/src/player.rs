use serde::{Deserialize, Serialize};

use crate::status::StatusMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub status_effects: StatusMap,
}

impl PlayerState {
    pub fn new(max_hp: i32, max_energy: i32) -> Self {
        Self {
            hp: max_hp,
            max_hp,
            block: 0,
            energy: max_energy,
            max_energy,
            status_effects: StatusMap::new(),
        }
    }

    /// Apply damage after block absorption. Returns actual HP lost.
    pub fn take_damage(&mut self, amount: i32) -> i32 {
        let after_block = (amount - self.block).max(0);
        self.block = (self.block - amount).max(0);
        self.hp -= after_block;
        after_block
    }

    pub fn gain_block(&mut self, amount: i32) {
        self.block += amount;
    }

    pub fn reset_block(&mut self) {
        self.block = 0;
    }

    /// Spend energy. Returns false (and does not deduct) if insufficient.
    pub fn spend_energy(&mut self, amount: i32) -> bool {
        if self.energy < amount {
            return false;
        }
        self.energy -= amount;
        true
    }

    /// Heal, capped at max_hp.
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_absorption() {
        let mut player = PlayerState::new(50, 3);
        player.gain_block(8);
        let lost = player.take_damage(12);
        assert_eq!(lost, 4);
        assert_eq!(player.hp, 46);
        assert_eq!(player.block, 0);
    }

    #[test]
    fn block_fully_absorbs() {
        let mut player = PlayerState::new(50, 3);
        player.gain_block(10);
        let lost = player.take_damage(6);
        assert_eq!(lost, 0);
        assert_eq!(player.hp, 50);
        assert_eq!(player.block, 4);
    }

    #[test]
    fn energy_spending() {
        let mut player = PlayerState::new(50, 3);
        assert!(player.spend_energy(2));
        assert_eq!(player.energy, 1);
        assert!(!player.spend_energy(2));
        assert_eq!(player.energy, 1);
    }

    #[test]
    fn heal_cap() {
        let mut player = PlayerState::new(50, 3);
        player.hp = 30;
        player.heal(100);
        assert_eq!(player.hp, 50);
    }
}
