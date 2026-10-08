//! Mega Diancie ex (PFL / ASC): Diamond Coat - this Pokémon takes 30 less
//! damage from attacks (after Weakness and Resistance). Garland Ray - discard
//! up to 2 Energy cards from this Pokémon; 120 damage for each card
//! discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaDiancieex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Among(AmongSpec { all_pokemon: false, which: AmongWhich::Any, max: Some(2), damage_per: 120, after_damage: true }), ..DiscardEnergySpec::DEFAULT }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), from_any_attack: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
