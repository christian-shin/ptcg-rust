//! Hop's Sandaconda (JTG 87): Rumble - 30; during your opponent's next turn, the Defending Pokémon can't retreat. Break Ground - 140; this attack also does 20 damage to each of your Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! A lasting retreat lock on the Defending Pokémon (Dusknoir's Shadow Bind); a ForEach over my Bench, each a Damage event of 20 with no Weakness or Resistance (Centiskorch SSP 28).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HopsSandaconda",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::ForEach(ForEachSpec { over: SlotSel::Bench(Who::Me), body: &[Step::new(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(20), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True }))] })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
