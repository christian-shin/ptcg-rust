//! Darumaka (PFL 15): Blaze Ball - 10+; this attack does 20 more damage for each [R] Energy attached to this Pokémon.
//!
//! Counted before the damage: the [R] Energy this Pokémon provides, 20 each (Golduck ASC 40's Hydro Pump).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Darumaka",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::FIRE)), &Num::Lit(20)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
