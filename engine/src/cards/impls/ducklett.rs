//! Ducklett (WHT 25): Firefighting - discard a [R] Energy from your opponent's Active Pokémon. Wing Attack - 20.
//!
//! The attacker chooses one Energy attached to the Defending Pokémon that provides [R] and it is discarded (a LeavePlay caused by
//! this attack; Farfetch'd's Mach Cut with a type); with none, nothing happens.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ducklett",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Chosen { pred: Pred::ProvidesType(ct::FIRE) }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
