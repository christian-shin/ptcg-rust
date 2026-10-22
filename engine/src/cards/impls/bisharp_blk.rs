//! Bisharp (BLK 65): Cut Up - 40. Finishing Blow - 60+; if your opponent's Active Pokémon already has any damage counters on it, this attack does 60 more damage.
//!
//! Adds 60 before the damage when the opponent's Active Pokémon is damaged (Larry's Rufflet ASC 173).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BisharpBLK",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(60, Cond::Slot(OPP_ACTIVE, SlotPred::Damaged))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
