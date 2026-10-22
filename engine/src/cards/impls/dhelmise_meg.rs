//! Dhelmise (MEG 18): Earthen Power - 30+; if you have a Stadium in play, this attack does 50 more damage.
//!
//! The Stadium in play (either player's) adds 50 before the damage (Fan Rotom SCR).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DhelmiseMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(50, Cond::StadiumInPlay(Pred::Any)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
