//! Deoxys (M4 33): Psy Protection - 80; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Pokémon that have
//! an Ability.
//!
//! Twinleaf: PREVENT_DAMAGE with `{ sourceHasAbility: true }` (damage only).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Deoxys3",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::HasAbility) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
