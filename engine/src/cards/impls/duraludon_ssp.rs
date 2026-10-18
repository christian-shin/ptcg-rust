//! Duraludon (SSP 129): Confront: 50. Duralubeam: 130; discard 2 Energy from this Pokémon.
//!
//! The 2 Energy are discarded from this Pokémon after the damage (LeavePlay events, this attack their cause): up to 2 cards,
//! never more than the 2 Energy units the text names (id2127's pattern for multi-Energy cards).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DuraludonSSP",
    // Duralubeam: discard 2 Energy from this Pokémon.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
