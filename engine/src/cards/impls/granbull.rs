//! Granbull (PFL 38): Bite - 50. Finishing Blow - 90+; if your opponent's Active Pokémon already has any damage counters on it, this attack does 90 more damage.
//!
//! Adds 90 before the damage when the opponent's Active Pokémon is damaged (Larry's Rufflet ASC 173).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Granbull",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(90, Cond::Slot(OPP_ACTIVE, SlotPred::Damaged))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
