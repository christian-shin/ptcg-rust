//! Lampent (TWM 37): Live Coal - 20. Burn It All Up - 60; discard all Energy from this Pokémon.
//!
//! Every Energy card on the Pokémon is discarded after the damage: a LeavePlay per card (D1), cause: this attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LampentTWMPool",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::This), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
