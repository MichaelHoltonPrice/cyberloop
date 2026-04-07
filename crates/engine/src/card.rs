use serde::{Deserialize, Serialize};

use crate::card_ids::CardId;
use crate::class::{CardTag, Class};
use crate::ids::SubclassId;
use crate::status::StatusType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl Rarity {
    pub fn label(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Legendary => "Legendary",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Spell,
    Consumable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    SingleEnemy,
    AllEnemies,
    Player,
    RandomEnemy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    DealDamage {
        target: Target,
        amount: i32,
    },
    GainBlock {
        amount: i32,
    },
    ApplyStatus {
        target: Target,
        status: StatusType,
        stacks: i32,
    },
    DrawCards {
        count: usize,
    },
    GainEnergy {
        amount: i32,
    },
    Heal {
        amount: i32,
    },
    /// Lose HP directly (bypasses block, no modifiers).
    LoseHp {
        amount: i32,
    },
    /// Deal damage only if the player took damage last enemy turn (for Riposte).
    DealDamageIfDamagedLastTurn {
        target: Target,
        amount: i32,
    },
    /// Deal amount damage; if target has any debuff, deal amount + bonus instead.
    DealDamageIfTargetDebuffed {
        target: Target,
        amount: i32,
        bonus: i32,
    },
    /// Draw N cards, then discard M random cards from hand.
    DrawAndDiscard {
        draw: usize,
        discard: usize,
    },
    /// Remove block from the target enemy before dealing damage.
    RemoveEnemyBlock {
        target: Target,
        amount: i32,
    },
    /// Gain extra block if the player took damage last enemy turn.
    GainBlockIfDamagedLastTurn {
        amount: i32,
    },
    /// Gain block equal to missing HP, capped at max.
    BlockEqualMissingHp {
        max: i32,
    },
    /// Deal amount damage; if enemy has block > 0, deal amount + bonus instead.
    DealDamageIfEnemyBlocked {
        target: Target,
        amount: i32,
        bonus: i32,
    },
    /// Deal damage equal to the player's current block.
    DealDamageEqualBlock {
        target: Target,
    },
    /// Deal per_card damage for each card played this turn (including this one).
    DealDamagePerCardPlayed {
        target: Target,
        per_card: i32,
    },
    /// Deal damage only if the target has Marked stacks > 0.
    DealDamageIfMarked {
        target: Target,
        amount: i32,
    },
    /// Apply per_card stacks of status for each card played this turn (including this one).
    ApplyStatusPerCardPlayed {
        target: Target,
        status: StatusType,
        per_card: i32,
    },
    /// Deal per_stack damage for each Marked stack on target, then remove all Marked.
    DealDamagePerMarkedStack {
        target: Target,
        per_stack: i32,
    },
    /// Gain block equal to player's stacks of `status` × `multiplier`.
    GainBlockPerStatusStack {
        status: StatusType,
        multiplier: i32,
    },
    /// Draw 1 card for each alive enemy that has the given status.
    DrawPerEnemyWithStatus {
        status: StatusType,
    },
    // ── New effects for Phase 3 cards ──────────────────────────────────
    /// Brace: gain `amount` block; if already have block, gain `bonus_amount` instead.
    GainBlockConditional {
        amount: i32,
        bonus_amount: i32,
    },
    /// Unbreakable: block cannot go below `amount` this combat.
    BlockFloor {
        amount: i32,
    },
    /// Iron Fortress: retain up to `amount` block permanently.
    RetainBlockPartial {
        amount: i32,
    },
    /// Expose Weakness: double stacks of `status` on `target`.
    DoubleStatus {
        target: Target,
        status: StatusType,
    },
    /// Coup de Grace: deal `base_damage` + `per_stack` per Marked stack.
    DealDamageScaledByMarked {
        target: Target,
        base_damage: i32,
        per_stack: i32,
    },
    /// Deal base + per_debuff for each distinct debuff type on the target.
    DealDamagePerDebuffOnTarget {
        target: Target,
        base: i32,
        per_debuff: i32,
    },
    /// Deal base + scaling per 5 missing player HP, capped.
    DealDamageScaledByMissingHp {
        target: Target,
        base: i32,
        per_5_missing: i32,
        cap: i32,
    },
    /// Deal per_stack damage for each stack of a player status (e.g. Rage).
    DealDamagePerPlayerStatus {
        target: Target,
        status: StatusType,
        per_stack: i32,
    },
    /// Enter a stance (removes any other active stance first).
    EnterStance {
        stance: StatusType,
    },
    /// Deal base damage; if player has >= ki_cost Ki, spend it and deal bonus instead.
    DealDamageSpendKi {
        target: Target,
        base: i32,
        ki_cost: i32,
        bonus: i32,
    },
    /// Gain base block; if player has >= ki_cost Ki, spend it and gain bonus instead.
    GainBlockSpendKi {
        base: i32,
        ki_cost: i32,
        bonus: i32,
    },
    /// Gain block equal to Ki stacks * multiplier (does NOT spend Ki).
    GainBlockPerKiStack {
        multiplier: i32,
    },
    /// Deal base + scaling per 10% enemy HP missing (execute damage).
    DealDamageScaledByMissingHpEnemy {
        target: Target,
        base: i32,
        per_10_percent_missing: i32,
    },
    /// Deal base damage; if player has >= smite_cost Smite, spend it and deal bonus instead.
    DealDamageSpendSmite {
        target: Target,
        base: i32,
        smite_cost: i32,
        bonus: i32,
    },
    /// Gain base block; if player has >= smite_cost Smite, spend it and gain bonus instead.
    GainBlockSpendSmite {
        base: i32,
        smite_cost: i32,
        bonus: i32,
    },
    /// Deal base damage; if player has >= artifice_cost Artifice, spend it and deal bonus instead.
    DealDamageSpendArtifice {
        target: Target,
        base: i32,
        artifice_cost: i32,
        bonus: i32,
    },
    // ── Druid Wild Shape effects ─────────────────────────────────────────
    /// Enter a Wild Shape form (clears other forms and concentration).
    EnterWildShape {
        form: StatusType,
    },
    /// Deal base damage; if player is in ANY Wild Shape form, deal base + bonus.
    DealDamageIfInWildShape {
        target: Target,
        base: i32,
        bonus: i32,
    },
    /// Deal base damage; if player has specific status, deal base + bonus.
    DealDamageWithFormBonus {
        target: Target,
        base: i32,
        status: StatusType,
        bonus: i32,
    },
    /// Gain base block; if player has specific status, gain base + bonus.
    GainBlockWithFormBonus {
        base: i32,
        status: StatusType,
        bonus: i32,
    },
    /// Draw base cards; if player has specific status, draw base + bonus.
    DrawCardsWithFormBonus {
        base_draw: usize,
        status: StatusType,
        bonus_draw: usize,
    },
    /// Deal damage only if player has the given status.
    DealDamageIfPlayerHasStatus {
        target: Target,
        status: StatusType,
        amount: i32,
    },
    /// Conditionally apply a status to the player if they have enough stacks of another status.
    ApplyStatusIfPlayerHasStatus {
        status: StatusType,
        stacks: i32,
        require_status: StatusType,
        require_stacks: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    pub id: CardId,
    pub name: String,
    pub rarity: Rarity,
    pub cost: i32,
    pub card_type: CardType,
    /// If true, spell goes to exhaust pile after play.
    pub exhaust: bool,
    pub effects: Vec<Effect>,
    pub description: String,
    /// Which playable class this card intrinsically belongs to. None means the
    /// card itself is not class-owned; reward-pool membership is declared
    /// separately in content.
    pub class: Option<Class>,
    /// Which sub-class theme or progression track this card belongs to.
    pub subclass: Option<SubclassId>,
    /// Semantic tags for class mechanic interactions.
    pub tags: Vec<CardTag>,
    /// Goes to bottom of draw pile instead of discard when played.
    pub recycle: bool,
    /// Stays in hand at end of turn; only 1 concentration card retained at a time.
    pub concentration: bool,
    /// Innate cards are drawn first at the start of combat.
    pub innate: bool,
    /// If true, this card is a milestone card (never in combat reward pool).
    pub milestone: bool,
    /// If true, this card is inherent (locked in deck, cannot be removed or swapped).
    pub inherent: bool,
}

impl CardDef {
    /// Does this card require a single enemy target?
    pub fn needs_single_target(&self) -> bool {
        self.effects.iter().any(|e| {
            matches!(
                e,
                Effect::DealDamage {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::ApplyStatus {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageIfDamagedLastTurn {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageIfTargetDebuffed {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::RemoveEnemyBlock {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageIfEnemyBlocked {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageEqualBlock {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamagePerCardPlayed {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageIfMarked {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::ApplyStatusPerCardPlayed {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamagePerMarkedStack {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DoubleStatus {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageScaledByMarked {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamagePerDebuffOnTarget {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageScaledByMissingHp {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamagePerPlayerStatus {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageSpendKi {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageScaledByMissingHpEnemy {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageSpendSmite {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageSpendArtifice {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageIfInWildShape {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageWithFormBonus {
                    target: Target::SingleEnemy,
                    ..
                } | Effect::DealDamageIfPlayerHasStatus {
                    target: Target::SingleEnemy,
                    ..
                }
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardInstance {
    pub def_id: CardId,
    pub upgraded: bool,
}
