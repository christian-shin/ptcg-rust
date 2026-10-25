//! Hop's Corviknight (JTG 108): Shoot Through - 50; this attack also does 50 damage to 1 of your opponent's Benched Pokémon. (Don't
//! apply Weakness and Resistance for Benched Pokémon.) Steel Wing - 150; during your opponent's next turn, this Pokémon takes 60 less
//! damage from attacks (after applying Weakness and Resistance).
//!
//! Shoot Through: exactly one Benched Pokémon is chosen at step D (none without a Bench), 50 damage with no Weakness or Resistance
//! after the damage. Steel Wing arms `Lasting::TakesLessDamage(60, Any)` on this Pokémon (Clamperl DRI 54's form).
use crate::spec::prelude::*;

const BENCH: SlotSel = SlotSel::Bench(Who::Opp);

pub static SPEC: CardSpec = CardSpec {
    class: "HopsCorviknight",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: BENCH,
                choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(1), &Num::SlotCount(BENCH, SlotPred::Any)), max: Num::Min(&Num::Lit(1), &Num::SlotCount(BENCH, SlotPred::Any)) }),
                what: EachWhat::Damage(DamageCalc::Put),
                amount: Num::Lit(50),
                ..EachSlotSpec::DEFAULT
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(60, DamageSource::Any) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
