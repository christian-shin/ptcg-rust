//! Wailord (JTG 41): Hydro Pump - 10+; this attack does 50 more damage for each [W] Energy attached to this Pokémon.
//!
//! Adds 50 per [W] Energy this Pokémon provides before the damage (Darumaka PFL 15's Blaze Ball; id2237).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Wailord",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::WATER)), &Num::Lit(50)), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
