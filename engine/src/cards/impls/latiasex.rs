//! Latias ex (SSP 76): Skyliner - your Basic Pokémon in play have no Retreat Cost. Eon Blade - 200; during your next turn this
//! Pokémon can't attack.
//!
//! Skyliner is a `RetreatCost` `Free` for your Basic Pokémon (`SlotPred::Basic`, the owner's side) while its Ability works,
//! read by the derived Retreat Cost of the Active Pokémon; "has no Retreat Cost" is a set cost, so it wins over "[C] more" effects
//! (D-13; JP FAQ Latias ex vs Mega Chandelure ex). Eon Blade arms a lasting `CANT_ATTACK` lock on this Pokémon until your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Latiasex",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Your Basic Pokémon have no Retreat Cost.
        modifier: Modifier::RetreatCost(RetreatCostSpec { change: CostChange::Free, subject: SlotPred::Basic, side: Side::Owner, ..RetreatCostSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
