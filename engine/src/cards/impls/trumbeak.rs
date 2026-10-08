//! Trumbeak (M5): Fly — 30; flip a coin. If tails, this attack does nothing.
//! If heads, during your opponent's next turn, prevent all damage from and
//! effects of attacks done to this Pokémon.
//!
//! Twinleaf FLIP_COIN_FOR_FLY: tails sets the attack damage to 0; heads
//! reduces a PreventDamageEffect and a PreventEffectsOfAttacksEffect (both
//! with the empty filter, target = the attacker's slot).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Trumbeak",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects }))],
            tails: &[Step::new(damage_is(Num::Lit(0)))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
