//! N's Reshiram (JTG / ASC): Powerful Rage - 20 damage for each damage
//! counter on this Pokémon. Virtuous Flame - 170.
//!
//! Twinleaf: `effect.damage = player.active.damage * 2` (the Active of the
//! attacking player).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsReshiram",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Mul(&Num::DamageOn(MY_ACTIVE), &Num::Lit(2)), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
