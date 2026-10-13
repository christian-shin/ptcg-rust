//! Diancie (SCR 86): Diffuse Reflection — 40 damage for each Special Energy attached to all of your opponent's Pokémon.
//! Power Gem — 60.
//!
//! Diffuse Reflection sets the damage to 40 times the Special Energy cards attached to the opponent's Pokémon, Active and
//! Bench (`EnergyUnit::SpecialEnergyCards`: a Special Energy card is one Special Energy, however much Energy it provides;
//! the "Energy provided" rule of RULES.md keeps these counts as cards).
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
