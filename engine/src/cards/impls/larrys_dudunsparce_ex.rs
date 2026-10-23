//! Larry's Dudunsparce ex (ASC 164): Work Rush - 80×; flip a coin for each Energy attached to this Pokémon. This attack
//! does 80 damage for each heads.
//!
//! One coin per Energy this Pokémon provides (id2237) before the damage, 80 per heads (`Flips::Count` over `Num::EnergyOn`,
//! Tauros CRI's count of flips).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LarrysDudunsparceEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec {
            flips: Flips::Count(Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits)),
            per_heads: PerHeads::DamageIs(80),
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
