//! Archaludon (M2 / PFL 75): Coated Attack — 120; during your opponent's
//! next turn, prevent all damage done to this Pokémon by attacks from Basic
//! Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE with `{ sourceStage: BASIC }`.
use crate::spec::prelude::*;
use crate::types::Stage;
pub static SPEC: CardSpec = CardSpec {
    class: "Archaludon@Archaludon M2",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Stage(Stage::Basic)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
