//! Genesect (M2 / PFL 8): Bug's Cannon — choose 1 of your opponent's Pokémon; 20 damage to it for each [G] Energy attached
//! to this Pokémon. Speed Attack — 110.
//!
//! Bug's Cannon: the chosen Pokémon (no cancel) takes a Damage event caused by the attack, even for 0 damage; Weakness
//! and Resistance apply only if it is the Active Pokémon (APR B-08). Two `Genesect` classes exist; this port is bound
//! to PFL.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Genesect@PFL",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::Provided(ct::GRASS)), &Num::Lit(20)), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
