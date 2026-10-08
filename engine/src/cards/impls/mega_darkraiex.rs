//! Mega Darkrai ex (M5 / PBL 48): Dusk Raid — 110+; 110 more if any of your
//! Benched Pokémon has damage counters. Abyss Eye — if the opponent's Active
//! is affected by a Special Condition, it is Knocked Out.
//!
//! Twinleaf clears the Active's Special Conditions before reducing the
//! (Mist-blockable) KnockOutOpponentEffect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaDarkraiex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(110), when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Damaged) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::If(IfSpec { cond: Cond::Slot(OPP_ACTIVE, SlotPred::HasCondition), yes: &[Step::new(Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::RemoveAll, cause: Cause::Direct, gate: Gate::None, when: Cond::True })), Step::new(Op::KnockOut(KnockOutSpec { target: OPP_ACTIVE, mode: KnockOutMode::Opponent, when: Cond::True }))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
