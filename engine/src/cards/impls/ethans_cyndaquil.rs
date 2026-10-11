//! Ethan's Cyndaquil (DRI 32): Ember: 30; discard an Energy from this Pokémon.
//!
//! The Energy is chosen and discarded after the damage (a LeavePlay event, this attack its cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EthansCyndaquil",
    // Ember: discard an Energy from this Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
