//! Palafin ex (TWM): Hero's Spirit — can only be put into play with Palafin's
//! Zero to Hero. Giga Impact — 250; during your next turn, this Pokémon
//! can't attack.
//!
//! Twinleaf: any EvolveEffect for this card throws CANNOT_EVOLVE (Zero to Hero
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
        lock: LockDecl { actions: &[LockedAction::Evolve], card: Pred::Any, except: Pred::False, error: "CANNOT_EVOLVE" },
        while_: &[LockWhile::CardIsSource],
        ability: false,
    }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
