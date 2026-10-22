//! Virizion (WHT 10): Giga Drain - 30; heal from this Pokémon the same amount of damage you did to your opponent's Active Pokémon.
//! Emerald Blade - 130; during your next turn, this Pokémon can't use attacks.
//!
//! Giga Drain heals the damage the attack did to the Defending Pokémon, as done, even when more than it had left (id462;
//! `Num::DamageDone`: 0 when the damage is prevented). Emerald Blade arms the lasting `CANT_ATTACK` lock on this Pokémon until your
//! next turn (Mega Lucario ex's Mega Brave without a name).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Virizion",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::DamageDone, clear_conditions: false }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
