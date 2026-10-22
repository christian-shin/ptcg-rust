//! Dhelmise (MEG 18): Earthen Power - 30+; if you have a Stadium in play, this attack does 50 more damage.
//!
//! "A Stadium in play" is either player's (the Stadium zone is shared): `Cond::StadiumInPlay`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DhelmiseMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(50, Cond::StadiumInPlay(Pred::Any)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
