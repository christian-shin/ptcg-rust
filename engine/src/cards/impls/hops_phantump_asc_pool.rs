//! Hop's Phantump (ASC 95): Splashing Dodge — 10; flip a coin, if heads, during your opponent's next turn, prevent all
//! damage from and effects of attacks done to this Pokémon.
//!
//! On heads the attack arms `Lasting::PreventDamage(Any)` (a `Prevent` over `Kind(Damage)`, step 6, APR C-16) and
//! `Lasting::PreventAttackEffects` (a `Prevent` naming no kind), each one ApplyEffect event, both stored on this Pokémon.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HopsPhantumpASCPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
