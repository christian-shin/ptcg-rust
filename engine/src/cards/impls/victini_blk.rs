//! Victini (BLK 12): V-Force - 120; if you have 4 or fewer Benched Pokémon, this attack does nothing.
//!
//! The damage is set to 0 before the damage when your Bench holds 4 or fewer Pokémon; the attack is still used (Hop's Cramorant's
//! Fickle Spitting).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VictiniBLK",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Nothing, hp: Num::Lit(0), when: Cond::Cmp(Num::BenchCount(Who::Me), CmpOp::Le, Num::Lit(4)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
