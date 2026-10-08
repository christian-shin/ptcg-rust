//! Gurdurr (TWM 104): Knuckle Punch — 20. Superpower — 50; you may do 30
//! more damage. If you do, this Pokémon also does 30 damage to itself.
//!
//! Twinleaf: a non-yielding ConfirmPrompt (WANT_TO_USE_ABILITY); on yes,
//! `effect.damage += 30` and a PutDamageEffect(30) on `player.active`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gurdurr",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(30), when: Cond::True })), Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(30), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True }))], no: &[] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
