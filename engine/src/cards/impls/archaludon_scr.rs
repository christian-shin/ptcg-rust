//! Archaludon (SCR 107 / PRE 70): Metal Bridge — all of your Pokémon that have
//! [M] Energy attached have no Retreat Cost. Iron Blaster — 160; during your
//! next turn this Pokémon can't attack.
//!
//! Twinleaf (set-stellar-crown/archaludon.ts): on any CheckRetreatCostEffect of
//! the owner (findCardList throws if the card is nowhere), unless blocked, a
//! CheckProvidedEnergy on the Active always runs; if this card is in play and
//! the Active provides [M], `cost = []`. Iron Blaster sets
//! `cannotAttackNextTurnPending` on the Active.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Archaludon@SCR|PRE",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
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
