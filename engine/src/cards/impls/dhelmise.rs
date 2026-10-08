//! Dhelmise (PBL / M5): Vengeful Anchor — 30+, 140 more if you have 4 or
//! more Pokémon with the Hide 'n' Sneak Ability in your discard pile.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dhelmise",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(140), when: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::HasAbilityNamed("Hide 'n' Sneak")), CmpOp::Ge, Num::Lit(4)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
