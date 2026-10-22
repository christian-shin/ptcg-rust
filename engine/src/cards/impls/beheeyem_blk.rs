//! Beheeyem (BLK 41): Calm Mind - heal 40 damage from this Pokémon. Psychic - 80+; this attack does 30 more damage for each Energy attached to your opponent's Active Pokémon.
//!
//! A heal on this Pokémon; Psychic adds 30 per Energy unit the Defending Pokémon provides before the damage (Jynx MEG 57).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BeheeyemBLK",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(heal_active(40)),
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
