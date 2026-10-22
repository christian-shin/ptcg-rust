//! Lampent (WHT 17): Fire Blast — 50; discard an Energy from this Pokémon.
//!
//! One Energy card of the player's choice leaves this Pokémon after the damage: a LeavePlay caused by the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LampentWHT",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
