//! Steven's Skarmory (DRI 142): Razor Wing - 30. Sonic Double - this attack does 50 damage to 2 of your opponent's Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! Exactly min(2, the opponent's Pokémon in play) targets (no cancel), each a Damage event of 50 caused by the attack;
//! Weakness and Resistance only for the Active Pokémon (APR B-08; Greninja ex TWM 106's Mirage Barrage).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensSkarmory",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), }), what: EachWhat::Damage(DamageCalc::Auto), amount: Num::Lit(50), ..EachSlotSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
