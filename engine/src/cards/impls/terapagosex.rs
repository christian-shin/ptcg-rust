//! Terapagos ex (SCR 128 / ASC 179, Tera): Unified Beatdown — 30× damage for each
//! of your Benched Pokémon; if you go second, you can't use this attack during
//! your first turn. Crown Opal — 180; during your opponent's next turn, prevent
//! all damage done to this Pokémon by attacks from Basic non-[C] Pokémon. Tera: as
//! long as this Pokémon is on your Bench, prevent all damage done to it by attacks.
//!
//! Rule: Unified Beatdown fails (CANNOT_USE_ATTACK) on turns 1 and 2, the second
//! player's first turn. Crown Opal arms a lasting `Prevent` over `Kind(Damage)`
//! (`DamageSource::BasicNonColorless`: the attacker is a Basic Pokémon with a
//! non-[C] type), read at step 6; its effects, the Special Conditions included, still
//! happen. The Tera rule is `TERA_RULE`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Terapagosex",
    passives: &[
        // Tera: no attack damage while Benched.
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) },
    ],
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                // If you go second, you can't use it during your first turn.
                Step::before_damage(Op::Fail(FailSpec { unless: Cond::Not(&Cond::Cmp(Num::Turn, CmpOp::Le, Num::Lit(2))), error: "CANNOT_USE_ATTACK" })),
                Step::before_damage(damage_is(Num::Mul(&Num::BenchCount(Who::Me), &Num::Lit(30)))),
            ],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::BasicNonColorless) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
