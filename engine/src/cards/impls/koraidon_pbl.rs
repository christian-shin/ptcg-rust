//! Koraidon (PBL 47): Battle Claw - 30+; if your opponent's Active Pokémon is an Evolution Pokémon, this attack does 30 more damage. Gaia Impact - 190; discard all Energy from this Pokémon.
//!
//! Battle Claw adds 30 when the opponent's Active is an Evolution Pokémon (not a Basic; Haxorus BLK 70). Gaia Impact discards every Energy of the Active Pokémon after the damage (Cramorant TWM 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KoraidonPBL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(30, Cond::Slot(OPP_ACTIVE, SlotPred::Not(&SlotPred::Basic)))),
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
