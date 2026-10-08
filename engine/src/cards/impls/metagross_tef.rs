//! Metagross (TEF): Meteor Mash — 60; during your next turn, this Pokémon's
//! Meteor Mash attack does 60 more damage. Luster Blast — 200; discard 2
//! Energy from this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Metagross@TEF",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: crate::types::ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::NextTurnBonus(NextTurnBonusSpec { attack: "Meteor Mash", bonus: 60 }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
