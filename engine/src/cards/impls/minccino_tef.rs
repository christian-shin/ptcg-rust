//! Minccino (TEF 136): Beat: 10. Cleaning Up: discard up to 2 Pokémon Tools from your opponent's Pokémon.
//!
//! The Tools are chosen among the opponent's Pokémon and discarded (LeavePlay events, this attack their cause; Mist Energy or
//! Hide 'n' Sneak on a Pokémon keep its Tool).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MinccinoTEF",
    attacks: &[
        AttackSpec { index: 0, steps: &[] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::OppTools { max: 2 }, ..DiscardEnergySpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
