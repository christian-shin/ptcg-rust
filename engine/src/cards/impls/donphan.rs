//! Donphan (CRI 45): No Reprieve - 20; during your next turn, attacks used by this Pokémon do 120 more damage to your
//! opponent's Active Pokémon (before applying Weakness and Resistance). Smashing Headbutt - 180; discard 2 Energy from this
//! Pokémon.
//!
//! No Reprieve is a `NextTurnBonus` armed by its use that changes every attack of this card during your next turn
//! (`BonusFor::Any`, `BonusChange::More(120)`: applied to the attack's damage before Weakness and Resistance when the
//! attack does damage; it ends when this Pokémon's effects end, APR C-15). Smashing Headbutt discards 2 Energy after the
//! damage (Metagross TEF 115).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Donphan",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
    }],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::NextTurnBonus(NextTurnBonusSpec { attack: 0, applies_to: BonusFor::Any, change: BonusChange::More(120) }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
