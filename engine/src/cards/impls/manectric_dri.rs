//! Manectric (DRI 76): Zap Kick - 30. Flash Impact - 120; this attack also does 40 damage to 1 of your Benched Pokémon (no Weakness or Resistance on a Benched Pokémon).
//!
//! A `DamageSlot` on a Benched Pokémon of mine that I pick, `DamageCalc::Put` (Diggersby POR 65 does it on each Benched Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ManectricDRI",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Me) }),
                    hp: Num::Lit(40),
                    target_damage_mul: 0,
                    calc: DamageCalc::Put,
                    when: Cond::True,
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
