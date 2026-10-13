//! Miraidon ex (TEF 122): Repulsion Bolt - 60+; if your opponent's Active Pokémon already has any damage counters on it, this
//! attack does 100 more damage. Cyber Drive - 220; during your next turn, this Pokémon can't use Cyber Drive.
//!
//! Repulsion Bolt adds 100 before the damage when the opponent's Active Pokémon is damaged. Cyber Drive arms a lasting lock on
//! this Pokémon that names the attack (`CANT_ATTACK` naming `NamedAttack::This`) until your next turn.
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
