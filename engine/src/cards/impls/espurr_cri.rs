//! Espurr (CRI 36): Buddy Attack - 10+; if you played Emma from your hand during this turn, this attack does 60 more damage.
//!
//! The played-this-turn record of the player (APR E-26): a card named Emma played from the hand adds 60 before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EspurrCRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(60, Cond::PlayedThisTurn(Who::Me, Pred::Name("Emma"))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
