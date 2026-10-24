//! Forretress (DRI 140): Iron Shake-Up - 20; you may move any amount of [M] Energy from your Pokémon to your other Pokémon in any way
//! you like. Hurricane of Needles - 80x; flip 4 coins. This attack does 80 damage for each heads.
//!
//! Iron Shake-Up: after the damage a MoveEnergy of any amount (none included) of the Energy cards that provide [M], each from one
//! of your Pokémon to another, chosen at step D (`MoveEnergyMode::AnyAmount`); a MoveEnergy event for each card, caused by the
//! attack. Hurricane of Needles: 4 coins before the damage, 80 for each heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Forretress",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::MoveEnergy(MoveEnergySpec { chooser: Who::Me, owner: Who::Me, mode: MoveEnergyMode::AnyAmount { only: Some(ct::METAL) } }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(Num::Lit(4)), per_heads: PerHeads::DamageIs(80), ..CoinSpec::DEFAULT }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
