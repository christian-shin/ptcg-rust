//! Keldeo ex (WHT 30): Gale Thrust - 30+; if this Pokémon moved from your Bench to the Active Spot this turn, this attack does 90 more damage. Sonic Edge - 120; this attack's damage isn't affected by any effects on your opponent's Active Pokémon.
//!
//! Gale Thrust adds 90 when `Cond::ThisMovedToActive` (Mega Lopunny ex PFL 84). Sonic Edge sets the `IgnoreDefenderEffects` attack flag (Mega Lopunny ex).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KeldeoEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(90, Cond::ThisMovedToActive)),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
