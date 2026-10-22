//! Deerling (TEF 16): Flop - 10. Leaf Litter Tackle - 40; discard a [G] Energy from this Pokémon.
//!
//! One Energy providing [G] is chosen on this Pokémon and discarded after the damage (Ethan's Cyndaquil's form with a type).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DeerlingTEF",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::GRASS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
