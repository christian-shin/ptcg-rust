//! Mega Charizard X ex (M2 / PFL): Inferno X — discard any amount of [R]
//! Energy from among your Pokémon; 90 damage for each card discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaCharizardXex@Mega Charizard X ex M2",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Among(AmongSpec { all_pokemon: true, which: AmongWhich::Provides(crate::types::ct::FIRE), max: None, damage_per: 90, after_damage: true }), ..DiscardEnergySpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
