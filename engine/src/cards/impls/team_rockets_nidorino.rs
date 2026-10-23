//! Team Rocket's Nidorino (DRI 118): Hammer In - 30. Horn Rend - 60+; if your opponent's Active Pokémon already has any
//! damage counters on it, this attack does 60 more damage.
//!
//! Adds 60 before the damage when the Defending Pokémon is damaged (Granbull PFL 38, Bisharp BLK 65).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsNidorino",
    attacks: &[AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(60, Cond::Slot(OPP_ACTIVE, SlotPred::Damaged)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
