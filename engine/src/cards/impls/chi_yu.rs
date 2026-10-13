//! Chi-Yu (PBL 59): Whirling Envy — 20+; if this Pokémon has 2 or more damage counters on it, 90 more damage. This attack's
//! damage isn't affected by Weakness.
//!
//! The attack sets the attack flag NoWeakness; the bonus reads the damage counters on this Pokémon, the attacker (the Active
//! Pokémon while it attacks).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ChiYu@PBL",
    // Whirling Envy: 90 more if this Pokémon has 2 or more damage counters; not affected by Weakness.
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
