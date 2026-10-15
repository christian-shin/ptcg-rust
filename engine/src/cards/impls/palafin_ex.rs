//! Palafin ex (PRE 151 / TWM 61): Hero's Spirit - put this Pokémon into play only with the effect of Palafin's Zero to Hero
//! Ability. Giga Impact - 250; during your next turn, this Pokémon can't attack.
//!
//! Rules (RULES.md "Evolution timing", decided 2026-10-09): the lock covers every EnterPlay, Evolve and Swap of this card by
//! any effect but Palafin's Zero to Hero (Grand Tree included; JP FAQ Palafin ex: Ditto's Surprising Transformation can't swap it
//! in); Zero to Hero's own swap passes. Giga Impact arms a `CANT_ATTACK` lasting lock on this Pokémon until your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PalafinEx",
attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] }],
    // Can only be put into play with Palafin's Zero to Hero.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Both,
        // Put into play, evolved into or swapped in by anything but Palafin's Zero to Hero (JP FAQ Palafin ex: Ditto's
        // Surprising Transformation can't swap it in, いいえ、できません).
        lock: LockDecl::on(
            EventPred::All(&[
                EventPred::Any(&[EventPred::Kind(EventKind::EnterPlay), EventPred::Kind(EventKind::Evolve), EventPred::Kind(EventKind::Swap)]),
                EventPred::This(Role::Card),
                EventPred::Not(&EventPred::Cause(CausePred::All(&[CausePred::Kind(crate::cause::CauseKind::Ability), CausePred::Card(Pred::HasAbilityNamed("Zero to Hero"))]))),
            ]),
            "CANNOT_EVOLVE",
        ),
        while_: &[],
        ability: false,
    }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
