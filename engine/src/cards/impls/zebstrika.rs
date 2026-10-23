//! Zebstrika (WHT 32): Smash Kick - 30. Electrobullet - 100; this attack also does 30 damage to 1 of your opponent's
//! Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! The 30 is the attack's damage to a chosen Benched Pokémon of the opponent (one Damage event, `DamageCalc::Put`: no
//! Weakness or Resistance, APR B-08; Stonjourner MEG 81).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zebstrika",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp) }),
            hp: Num::Lit(30),
            target_damage_mul: 0,
            calc: DamageCalc::Put,
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
