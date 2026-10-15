//! Stunfisk ex (ASC 114): Big Bite — 30; during your opponent's next turn the Defending Pokémon can't retreat. Flopping Trap — 100+;
//! if this Pokémon has any damage counters on it, this attack does 100 more damage.
//!
//! Big Bite is a retreat lock on the Defending Pokémon until the end of the opponent's next turn. Flopping Trap reads the attacking
//! Pokémon (the player's Active Pokémon, which is the one using the attack) for damage counters when the damage is calculated.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StunfiskEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] },
        // This Pokémon (the attacker, the Active Pokémon) has any damage counters on it.
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(100, Cond::Slot(MY_ACTIVE, SlotPred::Damaged)))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
