//! Marnie's Morpeko (DRI 137): Spiky Wheel - 20+, 40 more for each [D] Energy attached to this Pokémon.
//!
//! The count is the [D] Energy provided by this Pokémon (`EnergyUnit::Provided`: an Energy that provides every type, like Legacy
//! Energy, counts; JP FAQ Marnie's Morpeko).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesMorpeko",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::DARK)), &Num::Lit(40)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
