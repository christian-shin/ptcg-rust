//! Mega Slowbro ex (MEP): Shellnado Spin — 180; during your opponent's next
//! turn, if this Pokémon is damaged by an attack (even if Knocked Out), place
//! 12 damage counters on the Attacking Pokémon.
//!
//! Twinleaf: THIS_POKEMON_RETALIATES_ON_DAMAGE_DURING_OPPONENTS_NEXT_TURN
//! arms `retaliateOnDamageNextTurnPending = { damage: 120, attack, sourceCard,
//! attackerPlayerId }` on the attacker's Active (see `attack.rs`, AfterDamage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaSlowbroex",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(120) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
