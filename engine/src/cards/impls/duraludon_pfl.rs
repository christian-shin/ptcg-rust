//! Duraludon (PFL 74): Hyper Beam: 70; discard an Energy from your opponent's Active Pokémon.
//!
//! The Energy is chosen from the Defending Pokémon's attached cards and discarded after the damage (a LeavePlay event, this
//! attack its cause: Mist Energy or Hide 'n' Sneak refuse it). No Energy: nothing happens.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Duraludon@PFL",
    // Hyper Beam: discard an Energy from your opponent's Active Pokémon.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Opp, Zone::Attached(OPP_ACTIVE)),
                predicate: Pred::Energy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                purpose: Some(PromptPurpose::DiscardCard),
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Register(0), ..DiscardEnergySpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
