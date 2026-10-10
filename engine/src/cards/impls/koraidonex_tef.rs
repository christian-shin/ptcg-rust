//! Koraidon ex (TEF): Retribution Strike — 20+, 10 more for each damage counter on this Pokémon. Kaiser Tackle — 280;
//! this Pokémon does 60 damage to itself.
//!
//! Retribution Strike reads the counters on this Pokémon (the Active Pokémon, as it attacks). Kaiser Tackle's self
//! damage is a Damage event caused by the attack on this Pokémon (no Weakness or Resistance, APR B-08); this Pokémon
//! can be Knocked Out by it at the state check.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Koraidonex@TEF",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageOn(MY_ACTIVE), when: Cond::True })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(60)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
