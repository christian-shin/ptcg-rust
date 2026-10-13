//! N's Sigilyph (JTG 64): Psychic Sphere - 20. Victory Symbol - if you use this attack when you have exactly 1 Prize card
//! remaining, you win this game.
//!
//! Victory Symbol is an `EndGame` with the attacking player the winner, reached only with exactly 1 Prize card left.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsSigilyph",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::before_damage(Op::If(IfSpec {
                cond: Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Eq, Num::Lit(1)),
                yes: &[Step::new(Op::EndGame(EndGameSpec { winner: Who::Me }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
