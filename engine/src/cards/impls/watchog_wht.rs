//! Watchog (WHT 73): Focus Energy - during your next turn, this Pokémon's Hyper Fang attack's base damage is 240. Hyper
//! Fang - 80; flip a coin. If tails, this attack does nothing.
//!
//! Focus Energy is a `NextTurnBonus` armed by its use that changes this card's other attack, Hyper Fang, during your next
//! turn (`BonusFor::Attack(1)`, `BonusChange::BaseDamage(240)`: the attack's printed damage is 240; it ends when this
//! Pokémon's effects end, APR C-15). Hyper Fang's coin comes before the damage and tails sets it to 0 (Charcadet PBL 11).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WatchogWHT",
    attacks: &[AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { tails: &[Step::new(attack_does_nothing(Cond::True))], ..CoinSpec::DEFAULT }))] }],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::NextTurnBonus(NextTurnBonusSpec { attack: 0, applies_to: BonusFor::Attack(1), change: BonusChange::BaseDamage(240) }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
