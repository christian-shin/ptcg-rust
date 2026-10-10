//! Palafin (TEF): Vanguard Punch — 130; this Pokémon also does 10 damage to itself for each damage counter on it (read
//! before the main damage). Double Hit — 90x; flip 2 coins, 90 damage for each heads.
//!
//! The self damage is a Damage event caused by the attack on this Pokémon (no Weakness or Resistance, APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Palafin@TEF",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Slot(MY_ACTIVE),
                hp: Num::DamageOn(MY_ACTIVE),
                target_damage_mul: 0,
                calc: DamageCalc::Deal,
                when: Cond::True,
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(2), per_heads: PerHeads::DamageIs(90), ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
