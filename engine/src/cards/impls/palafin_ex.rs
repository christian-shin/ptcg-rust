//! Palafin ex (TWM): Hero's Spirit — can only be put into play with Palafin's
//! Zero to Hero. Giga Impact — 250; during your next turn, this Pokémon
//! can't attack.
//!
//! Rules (RULES.md "Evolution timing", decided 2026-10-09): the lock covers every EnterPlay and Evolve of
//! this card from any zone by any effect (Grand Tree included); Zero to Hero is a Swap and isn't covered.
//!
//! Twinleaf (retired; the old lock was from the hand only): any EvolveEffect for this card throws CANNOT_EVOLVE (Zero to Hero
//! moves it into play without one). It used to be lifted when the generic
//! ability-lock probe said blocked, which the Iron Thorns ex hand lock (phase
//! 4b) would have turned into a way to evolve it from the hand.
//! The TWM support print and Palafin exSAR PRE share the same Palafinex logic.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Palafinex",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn }))] }],
    // Can only be put into play with Palafin's Zero to Hero.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Both,
        lock: LockDecl::on(EventPred::All(&[EventPred::Any(&[EventPred::Kind(EventKind::EnterPlay), EventPred::Kind(EventKind::Evolve)]), EventPred::This(Role::Card)]), "CANNOT_EVOLVE"),
        while_: &[],
        ability: false,
    }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
