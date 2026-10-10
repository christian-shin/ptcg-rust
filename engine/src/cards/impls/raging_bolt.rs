//! Raging Bolt (SCR): Thunderburst Storm — 30 damage to 1 of the opponent's
//! Pokémon for each Energy attached to this Pokémon. Dragon Headbutt — 130.
//!
//! A mandatory pick among all of the opponent's Pokémon, then one Damage event for 30 times the Energy units this
//! Pokémon provides (a Special Energy providing 2 counts twice): on the Active Pokémon with Weakness and Resistance, on
//! a Benched one without (APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RagingBolt",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(30)), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
