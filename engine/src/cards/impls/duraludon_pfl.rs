//! Duraludon (PFL / M2 74): Hyper Beam — 70; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Twinleaf: DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON (no prompt when
//! the Active has no Energy card).
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
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Register(0), ..DiscardEnergySpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
