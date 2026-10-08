//! Dipplin (SCR 13): Coated Attack — 20; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Basic Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE with `{ sourceStage: BASIC }`. Two `Dipplin`
//! classes exist; this port is bound to SCR.
//!
//! Fixed (phase 4b, W4): printed data only, Dipplin SCR is a Stage 1 that
//! evolves from Applin (Twinleaf had it as a Basic).
use crate::spec::prelude::*;
use crate::types::Stage;
pub static SPEC: CardSpec = CardSpec {
    class: "Dipplin@SCR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Stage(Stage::Basic)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
