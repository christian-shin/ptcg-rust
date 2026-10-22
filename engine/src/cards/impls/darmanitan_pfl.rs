//! Darmanitan (PFL 16): Blaze Ball - 40+; this attack does 40 more damage for each [R] Energy attached to this Pokémon.
//!
//! Adds 40 per [R] Energy this Pokémon provides before the damage (Darumaka PFL 15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DarmanitanPFL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::FIRE)), &Num::Lit(40)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
