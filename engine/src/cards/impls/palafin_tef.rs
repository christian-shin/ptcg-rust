//! Palafin (TEF): Vanguard Punch — 130; this Pokémon also does 10 damage to
//! itself for each damage counter on it (DealDamageEffect of `active.damage`
//! on the Active, computed before the main damage). Double Hit — 90x; flip 2
//! coins (`effect.damage = 90 * heads`).
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
