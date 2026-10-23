//! Tangrowth (MEG 7): Absorb - 30; heal 30 damage from this Pokémon. Pumped-Up Whip - 120+; if this Pokémon has at least 2
//! extra Energy attached (in addition to this attack's cost), this attack does 140 more damage.
//!
//! Absorb heals this Pokémon after the damage (Applin DRI 16). Pumped-Up Whip adds 140 before the damage when the Energy
//! this Pokémon provides, minus the cost of the attack as the game checks it, is 2 or more (Electivire ex DRI 69's
//! High-Voltage Press).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tangrowth",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(30))] },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(more_damage_if(140, Cond::Cmp(Num::Sub(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::CostNow), CmpOp::Ge, Num::Lit(2))))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
