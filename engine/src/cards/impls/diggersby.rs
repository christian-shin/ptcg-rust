//! Diggersby (POR 65): Earthquake - 140; this attack also does 30 damage to each of your Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.) Whap Down - 100.
//!
//! A ForEach over my Bench, each a Damage event of 30 with no Weakness or Resistance (Centiskorch SSP 28).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Diggersby",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Bench(Who::Me), body: &[Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(30), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True }))] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
