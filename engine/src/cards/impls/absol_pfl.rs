//! Absol (PFL 63): Allure — draw 2 cards. Dark Cutter — 60.
//!
//! One Draw event of 2 cards after the damage (cause: the attack).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AbsolPFL",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
