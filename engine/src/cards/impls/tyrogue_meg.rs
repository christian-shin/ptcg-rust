//! Tyrogue (MEG 71): Pow-Pow Punching — 10+; flip a coin until you get tails. This attack does 30 more damage for each heads.
//!
//! The flips come before the damage; each heads adds 30 (`Flips::UntilTails`, `PerHeads::DamageAdd`), as Mega Kangaskhan ex MEG 104.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TyrogueMEG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageAdd(30), ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
