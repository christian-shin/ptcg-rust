//! Larry's Rufflet (ASC 173): Peck the Wound - 20+; if your opponent's Active Pokémon already has any damage counters on it, this attack does 80 more damage.
//!
//! Adds 80 before the damage when the opponent's Active Pokémon is damaged (Miraidon ex TEF 122).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LarrysRufflet",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(80, Cond::Slot(OPP_ACTIVE, SlotPred::Damaged))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
