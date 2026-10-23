//! Bewear (MEG 112): Knuckle Punch - 50. Hyper Lariat - 100+; flip 2 coins. If both of them are heads, this attack does 100 more damage.
//!
//! The 2 coins always flip before the damage; with 2 heads the damage grows by 100 (a coin sequence whose `then` adds the damage,
//! Miltank MEG 106's "both of them are heads").
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bewear",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::Lit(2)),
            then: &[Step::new(more_damage_if(100, Cond::Cmp(Num::Heads, CmpOp::Eq, Num::Lit(2))))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
