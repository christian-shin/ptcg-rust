//! Lucario (30C): Aura Sphere — 100, and 60 damage to 1 of your opponent's
//! Benched Pokémon.
//!
//! (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! Rule: after the damage, with a Benched Pokémon on the opponent's side, you choose one
//! and it takes a Damage event of 60 (cause: this attack; no Weakness or Resistance, APR
//! B-08; a prevention on it, the Tera rule's, still refuses it at step 6).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Lucario@30C",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(60), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
