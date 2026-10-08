//! Terapagos ex (SCR, Tera): Unified Beatdown — 30 for each of your Benched
//! Pokémon; if you go second, you can't use it during your first turn.
//! Crown Opal — 180; during your opponent's next turn, prevent all damage
//! done to this Pokémon by attacks from Basic non-[C] Pokémon.
//!
//! Twinleaf: Unified Beatdown throws CANNOT_USE_ATTACK whenever
//! `state.turn <= 2` (either player). Crown Opal arms PREVENT_DAMAGE with
//! `{ sourceStage: BASIC, sourceCardTypes: [every type but C] }`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Terapagosex",
    passives: &[
        // Tera: no attack damage while Benched.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) },
    ],
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                // If you go second, you can't use it during your first turn.
                Step::before_damage(Op::Fail(FailSpec { when: Cond::Cmp(Num::Turn, CmpOp::Le, Num::Lit(2)), error: "CANNOT_USE_ATTACK" })),
                Step::before_damage(damage_is(Num::Mul(&Num::BenchCount(Who::Me), &Num::Lit(30)))),
            ],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::BasicNonColorless) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
