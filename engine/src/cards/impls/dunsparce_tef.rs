//! Dunsparce (TEF): Gnaw — 10. Dig — 30; flip a coin, if heads, during your
//! opponent's next turn, prevent all damage from and effects of attacks done
//! to this Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE then PREVENT_EFFECTS_OF_ATTACKS (EffectOfAttack
//! effects targeting the attacker) arm `preventDamageNextTurnPending` /
//! `preventEffectsOfAttacksNextTurnPending` = `{}` on the attacker's Active.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dunsparce@TEF|PRE",
    // Dig: flip a coin, if heads, during your opponent's next turn, prevent all damage from and
    // effects of attacks done to this Pokémon.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            heads: &[
                Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })),
                Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects })),
            ],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
