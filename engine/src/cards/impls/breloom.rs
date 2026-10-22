//! Breloom (DRI 6): Damage Rush - 30+; flip a coin until you get tails. This attack does 50 more damage for each heads. Mega Drain - 90; heal 30 damage from this Pokémon.
//!
//! The flips come before the damage, 50 more per heads (`Flips::UntilTails` + `PerHeads::DamageAdd`, Mega Kangaskhan ex); Mega Drain heals this Pokémon after the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Breloom",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageAdd(50), ..CoinSpec::DEFAULT })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(heal_active(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
