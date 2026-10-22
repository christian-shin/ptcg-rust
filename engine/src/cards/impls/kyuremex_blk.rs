//! Kyurem ex (BLK 28): Slash - 50. Blizzard Burst - 130; this attack also does 10 damage to each of your opponent's Benched
//! Pokémon for each Prize card your opponent has taken. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! One Damage event per Benched Pokémon of the opponent, 10 times the Prize cards the opponent has taken, no Weakness or
//! Resistance (APR B-08); nothing when none was taken (Ting-Lu TWM 110).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KyuremEx",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Mul(&Num::PrizesTaken(Who::Opp), &Num::Lit(10)), when: Cond::Cmp(Num::PrizesTaken(Who::Opp), CmpOp::Gt, Num::Lit(0)), ..EachSlotSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
