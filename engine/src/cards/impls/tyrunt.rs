//! Tyrunt (POR 44): Get Angry - 20×; this attack does 20 damage for each damage counter on this Pokémon.
//!
//! The damage is 20 per damage counter: twice the HP of damage on this Pokémon (`Num::DamageOn` counts HP; Dwebble BLK 51).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tyrunt",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::DamageOn(MY_ACTIVE), &Num::Lit(2))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
