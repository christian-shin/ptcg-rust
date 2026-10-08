//! Petilil (BLK 6): Hide — flip a coin, if heads, during your opponent's next
//! turn, prevent all damage from and effects of attacks done to this Pokémon.
//! Leafage — 10.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PetililBLKPool",
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
