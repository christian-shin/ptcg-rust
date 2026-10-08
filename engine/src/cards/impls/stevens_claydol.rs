//! Steven's Claydol (DRI): Eerie Light — 20; your opponent's Active Pokémon is
//! now Confused. Clay Blast — 220; discard all Energy from this Pokémon.
//!
//! The Special Condition is an effect of the attack: Mist Energy and other
//! attack-effect protection prevent it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensClaydol",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused], Cause::Attack)),
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
