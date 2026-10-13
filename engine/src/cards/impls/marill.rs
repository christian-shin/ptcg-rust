//! Marill (MC): Hide — flip a coin; if heads, during your opponent's next
//! turn, prevent all damage from and effects of attacks done to this Pokémon.
//! Flop — 20.
//!
//! Hide flips one coin after the damage; on heads it arms two lasting effects on this Pokémon through the opponent's next turn:
//! `Lasting::PreventDamage(Any)` (a `Prevent` over damage, step 6, APR C-16) and `Lasting::PreventAttackEffects`, each one
//! ApplyEffect event. Tails does nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Marill",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
