//! Tranquill (BLK 72): Fly - 40; flip a coin. If tails, this attack does nothing. If heads, during your opponent's next
//! turn, prevent all damage from and effects of attacks done to this Pokémon.
//!
//! Trumbeak PBL 67's Fly: the coin comes before the damage; tails sets the damage to 0, heads arms the lasting prevention
//! of damage and of attack effects on this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tranquill",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) })), Step::new(Op::Arm(ArmSpec { what: Lasting::PreventAttackEffects }))],
            tails: &[Step::new(attack_does_nothing(Cond::True))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
