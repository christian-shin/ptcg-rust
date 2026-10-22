//! Bronzor (PBL 63): Mirror Attack — 10+; if your opponent's Active Pokémon is a [M] Pokémon, this attack does 30 more damage.
//!
//! The bonus reads the opponent's Active Pokémon's type as the game checks it (`SlotPred::TypeIs`, APR D-07).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BronzorPBL",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(30, Cond::Slot(OPP_ACTIVE, SlotPred::TypeIs(ct::METAL))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
