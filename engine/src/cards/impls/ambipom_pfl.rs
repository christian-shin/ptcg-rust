//! Ambipom (PFL 79): Slap — 50. Dual Tail — discard 2 Energy from this Pokémon, and this attack does 60 damage to each of 2 of your
//! opponent's Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! The 2 Energy cards go after the damage (LeavePlay events caused by the attack), then exactly min(2, the opponent's Pokémon in play)
//! targets are chosen (at step D, no cancel), each a Damage event of 60 caused by the attack (Weakness and Resistance only for the
//! Active Pokémon, APR B-08), as Kyurem SFA 54's Trifrost.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AmbipomPFL",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: SlotSel::Pokemon(Who::Opp),
                choose: Some(ChooseN {
                    chooser: Who::Me,
                    min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)),
                    max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)),
                }),
                what: EachWhat::Damage(DamageCalc::Auto),
                amount: Num::Lit(60),
                ..EachSlotSpec::DEFAULT
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
