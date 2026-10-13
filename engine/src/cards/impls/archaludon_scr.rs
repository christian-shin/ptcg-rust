//! Archaludon (SCR 107 / PRE 70): Metal Bridge — all of your Pokémon that have [M] Energy attached have no Retreat Cost.
//! Iron Blaster — 160; during your next turn this Pokémon can't attack.
//!
//! Metal Bridge is an Ability `RetreatCost` (`Free`) over the owner's Pokémon that provide [M]; it is set before the
//! "more" changes (APR D-13). Iron Blaster arms a lasting `CANT_ATTACK` lock on this Pokémon until the end of your next turn.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Archaludon@SCR|PRE",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Your Pokémon that have [M] Energy attached have no Retreat Cost.
        modifier: Modifier::RetreatCost(RetreatCostSpec {
            change: CostChange::Free,
            subject: SlotPred::Provides(ct::METAL),
            side: Side::Owner,
            ..RetreatCostSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
