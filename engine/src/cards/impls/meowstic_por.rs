//! Meowstic (POR 34): Perplex - your opponent's Active Pokémon is now Confused. Psychic - 30+; this attack does 30 more damage for each Energy attached to your opponent's Active Pokémon.
//!
//! Confusion by `inflict` (Darmanitan BLK 14 does Burned); the bonus counts Energy provided by the Defending Pokémon (id2237, Beheeyem BLK 41).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MeowsticPOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Confused])),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(OPP_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(30)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
