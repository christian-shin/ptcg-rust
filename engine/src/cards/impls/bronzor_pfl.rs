//! Bronzor (PFL 71): Iron Defense — flip a coin. If heads, during your opponent's next turn, prevent all damage done to this
//! Pokémon by attacks. Rollout — 30.
//!
//! One flip after the damage (this attack does none); on heads one ApplyEffect event arms a lasting `Prevent` over the damage of
//! attacks on this Pokémon through the opponent's next turn (step 6, APR C-16). Only damage: the effects of attacks still apply.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BronzorPFL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) }))], ..CoinSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
