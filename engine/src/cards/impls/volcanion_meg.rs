//! Volcanion (MEG 25): Singe - your opponent's Active Pokémon is now Burned. Backfire - 130; put 2 [R] Energy attached to this
//! Pokémon into your hand.
//!
//! Singe: a GainCondition(Burned) after the damage. Backfire: 2 Energy that provide [R] are put from this Pokémon into the hand
//! after the damage (a LeavePlay to the hand caused by the attack; Mega Greninja ex's Water Energy to the hand).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VolcanionMEG",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Burned]))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(MY_ACTIVE),
                selection: EnergySelection::Choose { count: 2, ty: ct::FIRE, up_to: false },
                to: EnergyDest::Hand,
                ..DiscardEnergySpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
