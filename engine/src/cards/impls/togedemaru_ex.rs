//! Togedemaru ex (ASC 149): Stun Needle - 20; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Spiky Rolling
//! - 80+; if this Pokémon used Spiky Rolling during your last turn, this attack does 80 more damage.
//!
//! Stun Needle: the coin is flipped after the damage. Spiky Rolling reads the player's last-attack record (the attack used during
//! their previous turn, by this Pokémon: a failed attack and a Knock Out reset leave none) and adds 80 before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TogedemaruEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(more_damage_if(80, Cond::AttackedLastTurn { who: Who::Me, by: Pred::Any, attacker: LastAttacker::This, attack: AttackPred::Named("Spiky Rolling") }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
