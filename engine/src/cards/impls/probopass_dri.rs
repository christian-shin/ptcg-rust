//! Probopass (DRI 98): Power Gem - 40. Mountain Drop - 70+; if a Stadium is in play, this attack does 70 more damage.
//!
//! A Stadium in either player's zone (Chi-Yu TWM's Ground Melter check); the Stadium stays in play.
use crate::spec::prelude::*;

const STADIUM_IN_PLAY: Cond = Cond::Any(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Stadium), Pred::Any), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Stadium), Pred::Any)]);

pub static SPEC: CardSpec = CardSpec {
    class: "ProbopassDRI",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(70, STADIUM_IN_PLAY)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
