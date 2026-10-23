//! Arven's Mabosstiff ex (DRI 139): Vigorous Tackle - 30+; if this Pokémon has no damage counters on it, this attack does 120 more
//! damage. Boss Headbutt - 210; during your next turn, this Pokémon can't use Boss Headbutt.
//!
//! The bonus is read before the damage (the attack's own damage can't be on this Pokémon yet): no damage counters on this Pokémon.
//! Boss Headbutt arms a lasting `CANT_ATTACK` lock on this Pokémon naming this attack until the end of your next turn (Koraidon ex
//! ASC): its other attack stays usable (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ArvensMabosstiffEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(120, Cond::Slot(SlotExpr::This, SlotPred::Not(&SlotPred::Damaged))))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
