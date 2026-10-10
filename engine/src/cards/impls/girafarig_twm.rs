//! Girafarig (TWM 83): Dual Headbutt — 30; this attack also does 10 damage to 1
//! of your Benched Pokémon. (Don't apply Weakness and Resistance for Benched
//! Pokémon.)
//!
//! Rule: after the damage, with a Benched Pokémon, you choose one and it takes a
//! Damage event of 10 (cause: this attack; the attacker's own Bench, so no Weakness,
//! Resistance or "does N more" applies, APR B-08), which a prevention on it (Tera
//! rule, Sacred Charm) still refuses at step 6.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Girafarig@TWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Me), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(10), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
