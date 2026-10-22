//! Barraskewda (DRI 63): Sharp Fin - 40. Dive - 60; flip a coin. If heads, during your opponent's next turn, prevent all damage from and effects of attacks done to this Pokémon.
//!
//! On heads two lasting effects are armed on this Pokémon: the damage prevention and the effects prevention (Cynthia's Feebas DRI 52's Undulate).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Barraskewda",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects }))], ..CoinSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
