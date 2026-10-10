//! Spewpa (POR 8): Hide — flip a coin. If heads, during your opponent's next
//! turn, prevent all damage from and effects of attacks done to this Pokémon.
//!
//! Rule: heads arms the lasting prevention (`Op::Arm`, one ApplyEffect event on
//! itself): a `Prevent` stored on the Pokémon (`lasting_prevents`) that stops every
//! damage and effect of the opponent's attacks during their next turn (damage at
//! step 6, APR C-16; the effects by the same rule as Mist Energy).
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
