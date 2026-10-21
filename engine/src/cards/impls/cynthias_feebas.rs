//! Cynthia's Feebas (DRI 52): Undulate - 10; flip a coin. If heads, during your opponent's next turn, prevent all damage
//! from and effects of attacks done to this Pokémon.
//!
//! On heads two lasting effects are armed on this Pokémon: the damage prevention and the effects prevention (Dunsparce TEF
//! 128's Dig).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasFeebas",
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
