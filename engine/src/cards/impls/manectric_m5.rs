//! Manectric (M5): Flashing Barrier — 50; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Evolution
//! Pokémon. Sonic Edge — 110; not affected by any effects on your
//! opponent's Active Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE(..., { sourceIsEvolution: true }) and
//! THIS_ATTACKS_DAMAGE_ISNT_AFFECTED_BY_EFFECTS (`ignoreDefenderEffects` on
//! the AttackEffect; phase 4b R7B: it used to add the damage straight to the
//! Active, skipping the attacker's effects too).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Manectric@PBL",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Evolution) }))] },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
