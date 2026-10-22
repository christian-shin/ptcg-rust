//! Tauros (DRI 152): Horn Attack — 30. Clean Hit — 50+; if your opponent's Active Pokémon is an Evolution Pokémon, this attack does
//! 50 more damage.
//!
//! The bonus reads the Defending Pokémon's top card: a Pokémon that isn't a Basic Pokémon (as Haxorus BLK 70).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TaurosDRI",
    attacks: &[AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(50, Cond::Slot(OPP_ACTIVE, SlotPred::Not(&SlotPred::Basic))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
