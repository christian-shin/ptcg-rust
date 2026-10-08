//! Chi-Yu (M5): Whirling Envy — 20+, 90 more if your Active Pokémon has 2 or
//! more damage counters; not affected by Weakness.
//!
//! Twinleaf checks `player.active.damage` (not necessarily this Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ChiYu@Chi-Yu M5",
    // Whirling Envy: 90 more if your Active Pokémon has 2 or more damage counters; not affected by Weakness.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(more_damage_if(90, Cond::Cmp(Num::DamageOn(MY_ACTIVE), CmpOp::Ge, Num::Lit(20)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
