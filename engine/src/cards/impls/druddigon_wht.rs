//! Druddigon (WHT 71): Shred - 40; this attack's damage isn't affected by any effects on your opponent's Active Pokémon.
//! Ambush - 90+; flip a coin. If heads, this attack does 60 more damage.
//!
//! Shred sets the attack flag `IgnoreDefenderEffects` before the damage (Crustle DRI 12); Ambush flips before the damage and
//! adds 60 on heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Druddigon",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(60, Cond::True))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
