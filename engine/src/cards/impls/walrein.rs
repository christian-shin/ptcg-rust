//! Walrein (SSP): Frigid Fangs — 60; during your opponent's next turn, Pokémon that have 2 or less Energy attached can't
//! attack (a player-level lasting effect: `Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::UseAttack), EventPred::EnergyAtMost(2)]), "BLOCKED_BY_EFFECT")))`, one ApplyEffect event on the player,
//! so it includes Pokémon that come into play later, and Walrein leaving the Active Spot doesn't end it: id2055).
//! Megaton Fall — 170; this Pokémon also does 50 damage to itself.
//!
//! The self damage is a Damage event caused by the attack on this Pokémon (no Weakness or Resistance, APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Walrein",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_opponent(&LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::UseAttack), EventPred::EnergyAtMost(2)]), "BLOCKED_BY_EFFECT"))) }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(self_damage(50))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
