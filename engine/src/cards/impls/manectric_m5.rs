//! Manectric (M5): Flashing Barrier — 50; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Evolution
//! Pokémon. Sonic Edge — 110; not affected by any effects on your opponent's
//! Active Pokémon.
//!
//! Flashing Barrier: one ApplyEffect event (`Op::Arm`) leaves a lasting `Prevent` over `Kind(Damage)` on this Pokémon,
//! read at step 6 of the damage (APR C-16) against the attacker's stage (an Evolution Pokémon); it ends with the
//! opponent's next turn. Sonic Edge sets the attack's ignore-the-defender's-effects flag: the Damage event skips the
//! defending side's modifiers and preventions (Shred), keeping the attacker's own.
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
