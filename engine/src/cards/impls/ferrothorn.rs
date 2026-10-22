//! Ferrothorn (WHT 69): Power Whip - this attack does 20 damage to 1 of your opponent's Pokémon for each Energy attached to this Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.) Metal Claw - 130.
//!
//! A chosen opponent's Pokémon takes 20 per Energy unit this Pokémon provides, `DamageCalc::Auto` (Weakness and Resistance only for the Active Pokémon, APR B-08; Cramorant TWM 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ferrothorn",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }), hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(20)), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
