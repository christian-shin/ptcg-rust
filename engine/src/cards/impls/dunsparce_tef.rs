//! Dunsparce (TEF): Gnaw — 10. Dig — 30; flip a coin, if heads, during your opponent's
//! next turn, prevent all damage from and effects of attacks done to this
//! Pokémon.
//!
//! On heads, two ApplyEffect events (`Op::Arm`) leave lasting `Prevent`s on this Pokémon: one over `Kind(Damage)` (any
//! attacker) and one over the effects of attacks (`EFFECTS_OF_OPP_ATTACKS`, never damage: APR C-17). Both end with the
//! opponent's next turn.
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
