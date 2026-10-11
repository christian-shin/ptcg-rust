//! Mega Charizard X ex (PFL 13): Inferno X - discard any amount of [R] Energy from among your Pokémon; 90 damage for each
//! card discarded.
//!
//! An attack's "any amount" may be 0 (id2301). The Energy is chosen before the damage, each card is a LeavePlay (D1) that
//! happens after the damage is dealt (the count sets the damage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaCharizardXex@PFL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Among(AmongSpec { all_pokemon: true, which: AmongWhich::Provides(crate::types::ct::FIRE), max: None, damage_per: 90, after_damage: true }), ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
