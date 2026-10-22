//! Snom (MEG 42): Hide - flip a coin. If heads, during your opponent's next turn, prevent all damage from and effects of attacks done to this Pokémon.
//!
//! On heads, two ApplyEffect events leave lasting `Prevent`s on this Pokémon: one over damage, one over the effects of
//! attacks (APR C-17; Dunsparce TEF 128).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SnomMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Coin(CoinSpec {
                    heads: &[
                        Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })),
                        Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects })),
                    ],
                    ..CoinSpec::DEFAULT
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
