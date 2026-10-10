//! Centiskorch (SSP): Billowing Heat Wave — 130; also 30 damage to each of
//! your Benched Pokémon (no Weakness or Resistance, APR B-08). Heat Blast — 80.
//!
//! Each 30 is its own Damage event on one of your Benched Pokémon: a "takes N less" and a prevention of damage on a
//! Benched Pokémon of yours apply (rule-B09).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Centiskorch",
    // Billowing Heat Wave: also 30 damage to each of your Benched Pokémon (no Weakness or Resistance).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
            over: SlotSel::Bench(Who::Me),
            body: &[Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(30), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True }))],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
