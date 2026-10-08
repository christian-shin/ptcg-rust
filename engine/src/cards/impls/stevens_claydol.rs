//! Steven's Claydol (DRI): Eerie Light — 20; your opponent's Active Pokémon is
//! now Confused. Clay Blast — 220; discard all Energy from this Pokémon.
//!
//! Twinleaf: the confusion is applied on AFTER_ATTACK through
//! ADD_CONFUSION_TO_PLAYER_ACTIVE (an AddSpecialConditionsPowerEffect, so it is
//! not an attack effect); Clay Blast discards via DISCARD_ALL_ENERGY_FROM_POKEMON.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensClaydol",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused], Cause::Ability)),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
