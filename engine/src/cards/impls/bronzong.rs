//! Bronzong (TEF 69): Evolution Jammer - 30; during your opponent's next turn, they can't play Pokémon from their
//! hand to evolve their Pokémon. Super Psy Bolt - 100.
//!
//! A lasting lock on the opponent over `Evolve` events whose source is the hand (`Lasting::Lock`, D14), placed by the
//! attack's ApplyEffect and ending with the opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Bronzong",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::Source(RulesZone::Hand), EventPred::Owner(Who::Opp)]), "BLOCKED_BY_EFFECT"))) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
