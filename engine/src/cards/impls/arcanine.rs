//! Arcanine (DRI 28): Flare - 50. Punishing Fang - 100+; if your opponent's Active Pokémon is a [D] Pokémon, this attack does 100
//! more damage.
//!
//! The bonus reads the opponent's Active Pokémon's type as the game checks it (`SlotPred::TypeIs`, APR D-07; Bronzor PBL 63).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Arcanine",
    attacks: &[AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(100, Cond::Slot(OPP_ACTIVE, SlotPred::TypeIs(ct::DARK))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
