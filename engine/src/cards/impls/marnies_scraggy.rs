//! Marnie's Scraggy (DRI 132): Crunch - 50; discard an Energy from your opponent's Active Pokémon.
//!
//! The Energy is chosen and discarded after the damage (a LeavePlay caused by the attack: Duraludon PFL 74's Hyper Beam); no
//! Energy: nothing happens.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesScraggy",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Opp, Zone::Attached(OPP_ACTIVE)),
                    predicate: Pred::Energy,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Register(0), ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
