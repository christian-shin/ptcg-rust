//! Cynthia's Milotic (DRI 53): Aqua Split - 60; this attack also does 30 damage to 2 of your opponent's Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! Exactly min(2, the opponent's Benched Pokémon) targets (no cancel), each a Damage event of 30 caused by the attack with no Weakness or Resistance (`DamageCalc::Put`, Hydreigon ex).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasMilotic",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Bench(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Bench(Who::Opp), SlotPred::Any)), }), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(30), ..EachSlotSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
