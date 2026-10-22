//! Seaking (PBL 14): Hydro Jet — this attack does 30 damage to 1 of your opponent's Pokémon for each [W] Energy attached to this
//! Pokémon (don't apply Weakness and Resistance for Benched Pokémon).
//!
//! The chosen Pokémon takes a Damage event caused by the attack, of 30 times the [W] Energy this Pokémon provides (id2237);
//! Weakness and Resistance apply only if it is the Active Pokémon (APR B-08). As Genesect PFL 8's Bug's Cannon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SeakingPBL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
            hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::WATER)), &Num::Lit(30)),
            target_damage_mul: 0,
            calc: DamageCalc::Auto,
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
