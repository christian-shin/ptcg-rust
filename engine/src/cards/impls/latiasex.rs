//! Latias ex (SSP): Skyliner - your Basic Pokémon in play have no Retreat
//! Cost. Eon Blade - 200; during your next turn this Pokémon can't attack.
//!
//! Twinleaf checks only whether the retreating (Active) Pokémon is Basic,
//! and only while this Latias ex is the top Pokémon of one of the owner's
//! in-play slots.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Latiasex",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Your Basic Pokémon have no Retreat Cost.
        modifier: Modifier::RetreatCost(RetreatCostSpec { change: CostChange::Free, subject: SlotPred::Basic, side: Side::Owner, ..RetreatCostSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
