//! Deerling (TEF 16): Flop - 10. Leaf Litter Tackle - 40; discard a [G] Energy from this Pokémon.
//!
//! One Energy that provides [G] is paid from this Pokémon after the damage (a LeavePlay caused by the attack; Mega Greninja ex's
//! Water Energy payment, to the discard pile).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DeerlingTEF",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
            target: SlotTarget::Slot(MY_ACTIVE),
            selection: EnergySelection::Choose { count: 1, ty: ct::GRASS, up_to: false },
            ..DiscardEnergySpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
