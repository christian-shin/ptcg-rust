//! Flittle (SSP): Splashing Dodge - 10; flip a coin, if heads, during your
//! opponent's next turn, prevent all damage from and effects of attacks done
//! to this Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE then PREVENT_EFFECTS_OF_ATTACKS on heads.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Flittle",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventEffects }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
