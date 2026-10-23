//! Mabosstiff (PBL 58): Bite - 60. Plunging Headbutt - 210; during your opponent's next turn, this Pokémon takes 100 more
//! damage from attacks (after applying Weakness and Resistance).
//!
//! `Lasting::ThisTakesMoreDamage(100)` armed on this Pokémon: the mirror of Carkol ASC 119's "takes less damage" (the same
//! lasting entry with the amount negated, in force during the opponent's next turn after Weakness and Resistance, gone when
//! this Pokémon moves to the Bench, APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mabosstiff",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::ThisTakesMoreDamage(100) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
