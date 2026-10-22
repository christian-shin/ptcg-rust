//! Shieldon (PBL 61): Smithereen Smash - 50; discard an Energy from your opponent's Active Pokémon.
//!
//! One chosen Energy of the Defending Pokémon after the damage (Floatzel DRI 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shieldon",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
