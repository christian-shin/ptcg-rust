//! Slakoth (SSP): Take It Easy — heal 60 damage from this Pokémon. During
//! your next turn, this Pokémon can't retreat.
//!
//! Twinleaf: HEAL_X_DAMAGE_FROM_THIS_POKEMON (a HealTargetEffect on
//! `player.active`) then BLOCK_SELF_RETREAT (a SelfPreventRetreatEffect whose
//! target is the attacker since phase 4b, so Mist Energy on the Defending
//! Pokémon no longer blocks it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slakoth",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(60)), Step::after_damage(Op::Arm(ArmSpec { what: Lasting::SelfCannotRetreat }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
