//! Raboot (MEG 27): Jumping Kick - this attack does 40 damage to 1 of your opponent's Pokémon (no Weakness or Resistance on a Benched Pokémon).
//!
//! One `DamageSlot` on a Pokémon the attacker picks among the opponent's, `DamageCalc::Auto` (Cramorant TWM 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RabootMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
                    hp: Num::Lit(40),
                    target_damage_mul: 0,
                    calc: DamageCalc::Auto,
                    when: Cond::True,
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
