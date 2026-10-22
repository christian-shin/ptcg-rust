//! Ducklett (WHT 25): Firefighting - discard a [R] Energy from your opponent's Active Pokémon. Wing Attack - 20.
//!
//! One Energy card of the Defending Pokémon that provides [R] as the game checks it is chosen by the attacker and discarded after
//! the damage (a LeavePlay caused by the attack): a Prism Energy counts as [R] only on a Basic Pokémon (id2194; Deerling's payment).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ducklett",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
            target: SlotTarget::Slot(OPP_ACTIVE),
            selection: EnergySelection::Choose { count: 1, ty: ct::FIRE, up_to: false },
            ..DiscardEnergySpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
