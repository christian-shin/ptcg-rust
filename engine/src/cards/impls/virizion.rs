//! Virizion (WHT 10): Giga Drain - 30; heal from this Pokémon the same amount of damage you did to your opponent's Active Pokémon.
//! Emerald Blade - 130; during your next turn, this Pokémon can't use attacks.
//!
//! Giga Drain heals this Pokémon (a RemoveCounters event caused by the attack) by the damage this attack did to the Defending Pokémon
//! after Weakness, Resistance and every effect (`Num::DamageDone`; id462: the whole damage counts even when it is more than the
//! Defending Pokémon had left); damage that was prevented heals nothing. Emerald Blade arms the lasting lock on this Pokémon over
//! every attack through the end of your next turn (Ceruledge SSP).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Virizion",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::DamageDone, clear_conditions: false }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
