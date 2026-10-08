//! Gouging Fire ex (TEF): Heat Blast — 60. Blaze Blitz — 260; this Pokémon
//! can't use Blaze Blitz again until it leaves the Active Spot
//! (PREVENT_THIS_ATTACK_UNTIL_LEAVES_ACTIVE: a PreventAttackUntilLeavesActive
//! EffectOfAttack that sets `source.blockedAttackNameUntilLeavesActive`; its
//! target is the attacker since phase 4b, so Empoleon ex / Mist Energy on the
//! Defending Pokémon no longer prevent it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GougingFireex",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::BlockThisAttackUntilLeavesActive }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
