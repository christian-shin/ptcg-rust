//! Dwebble (BLK 51): Flail — 10×; this attack does 10 damage for each damage counter on this Pokémon. Dig Claws — 20.
//!
//! The damage is the damage already on this Pokémon (a counter is 10 HP of damage), set before the damage (APR B-02).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DwebbleBLK",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::DamageOn(MY_ACTIVE)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
