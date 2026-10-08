//! Spewpa (POR / M3): Hide — flip a coin; if heads, during your opponent's
//! next turn prevent all damage and effects from attacks done to this
//! Pokémon.
//!
//! Twinleaf: FLIP_COIN_TO_PREVENT_DAMAGE_AND_EFFECTS_DURING_OPPONENTS_NEXT_TURN.
//! Phase 4b: heads used to call only PREVENT_DAMAGE; it now also calls
//! PREVENT_EFFECTS_OF_ATTACKS, like Petilil's Hide.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Spewpa@POR",
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
