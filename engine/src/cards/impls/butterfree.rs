//! Butterfree (JTG 3): Scale Hurricane - 60x; flip 4 coins. This attack does 60 damage for each heads. If at least 2 of them are
//! heads, your opponent's Active Pokémon is now Paralyzed.
//!
//! The 4 coins flip before the damage (60 per heads, `PerHeads::DamageIs`); the sequence's `then` runs after the flips: with 2 or more
//! heads the Defending Pokémon is Paralyzed (a Special Condition event caused by the attack; a coin sequence applies its damage
//! first, then `then`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Butterfree",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::Lit(4)),
            per_heads: PerHeads::DamageIs(60),
            then: &[Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::Heads, CmpOp::Ge, Num::Lit(2)), yes: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], no: &[] }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
