//! Dipplin (SCR 13): Coated Attack — 20; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Basic Pokémon.
//!
//! One ApplyEffect event (`Op::Arm`) leaves a lasting `Prevent` over `Kind(Damage)` on this Pokémon, read at step 6 of
//! the damage (APR C-16) against the attacker's stage (Basic); it ends with the opponent's next turn. Dipplin SCR is a
//! Stage 1 that evolves from Applin.
use crate::spec::prelude::*;
use crate::types::Stage;
pub static SPEC: CardSpec = CardSpec {
    class: "Dipplin@SCR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Stage(Stage::Basic)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
