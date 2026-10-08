//! Diancie (SCR): Diffuse Reflection — 40 damage for each Special Energy
//! attached to all of your opponent's Pokémon. Power Gem — 60.
//!
//! Twinleaf counts Special Energy cards in `cards` of every opponent slot
//! (bench, then Active) and sets `effect.damage = 40 * count`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Diancie",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::SpecialEnergyCards), &Num::Lit(40)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
