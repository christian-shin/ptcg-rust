//! Archen (WHT 50): Acrobatics - 30+; flip 2 coins. This attack does 30 more damage for each heads.
//!
//! The 2 coins are flipped before the damage and each heads adds 30 (`PerHeads::DamageAdd`, Mega Kangaskhan ex MEG 104). Archen
//! evolves from the Antique Plume Fossil (its data names it, as Tyrunt names the Antique Jaw Fossil).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Archen",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(2)), per_heads: PerHeads::DamageAdd(30), ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
