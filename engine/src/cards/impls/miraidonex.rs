//! Miraidon ex (TEF): Repulsion Bolt — 60+; 100 more if the opponent's Active
//! has damage counters. Cyber Drive — 220; THIS_POKEMON_CANNOT_USE_THIS_ATTACK_NEXT_TURN.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Miraidonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(100, Cond::Slot(OPP_ACTIVE, SlotPred::Damaged)))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
