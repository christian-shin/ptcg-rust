//! Feebas (TWM): Flail — 10x; `effect.damage = player.active.damage` (the
//! damage on the Active in HP, i.e. 10 per damage counter).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Feebas@Feebas TWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::DamageOn(MY_ACTIVE), when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
