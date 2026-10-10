//! Trumbeak (PBL 67): Fly — 30; flip a coin. If tails, this attack does nothing.
//! If heads, during your opponent's next turn, prevent all damage from and effects
//! of attacks done to this Pokémon.
//!
//! Rule: the coin comes before the damage. Tails sets the damage to 0; heads arms
//! the lasting prevention (`Op::Arm` x2, ApplyEffect events on itself): a `Prevent`
//! stored on the Pokémon that stops every damage (step 6, APR C-16) and effect of the
//! opponent's attacks during their next turn.
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
