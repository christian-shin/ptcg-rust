//! Deoxys (M4 33): Psy Protection - 80; during your opponent's next turn, prevent all damage done to this Pokémon by
//! attacks from Pokémon that have an Ability.
//!
//! `Lasting::PreventDamage(HasAbility)`: one ApplyEffect event arms a `Prevent` over `Kind(Damage)` stored on this
//! Pokémon (`Slot::lasting_prevents`); it is read at step 6 of the damage calculation (APR C-16) and never stops
//! effects.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Deoxys3",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::HasAbility) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
