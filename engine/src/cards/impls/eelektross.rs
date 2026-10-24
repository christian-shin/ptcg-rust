//! Eelektross (BLK 32): Thunder Fang - 60; flip a coin. If heads, your opponent's Active Pokémon is now Paralyzed. Buzz Flip - 100x;
//! flip 4 coins. This attack does 100 damage for each heads.
//!
//! Thunder Fang: the coin after the damage, Paralyzed on heads (Mareep CRI 27). Buzz Flip: 4 coins before the damage, 100 for each
//! heads (`PerHeads::DamageIs`, Butterfree JTG 3).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Eelektross",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(inflict(&[SpecialCondition::Paralyzed]))], ..CoinSpec::DEFAULT }))] },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(4)), per_heads: PerHeads::DamageIs(100), ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
