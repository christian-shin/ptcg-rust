//! Mega Slowbro ex (MEP): Shellnado Spin — 180; during your opponent's next
//! turn, if this Pokémon is damaged by an attack (even if Knocked Out), place
//! 12 damage counters on the Attacking Pokémon.
//!
//! `Lasting::Retaliate(120)` (an ApplyEffect event on this Pokémon): during the opponent's next turn each Damage event
//! an attack does to it, even one that Knocks it Out, records the trap; after that attack's damage the 12 counters are a
//! PlaceCounters event with Shellnado Spin as its cause on the Attacking Pokémon wherever it is now (id534, id2371),
//! so Mist Energy or Hide 'n' Sneak on it refuses them (id2408, id1958).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaSlowbroex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(120) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
