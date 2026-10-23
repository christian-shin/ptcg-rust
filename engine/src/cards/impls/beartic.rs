//! Beartic (BLK 26): Continuous Headbutt - 50×; flip a coin until you get tails. This attack does 50 damage for each heads.
//! Sheer Cold - 150; during your opponent's next turn, the Defending Pokémon can't use attacks.
//!
//! `Flips::UntilTails` + `PerHeads::DamageIs(50)` before the damage (Baltoy CRI 46); Sheer Cold arms the lasting
//! `CANT_ATTACK` lock on the Defending Pokémon until the end of the opponent's next turn (Cubchoo BLK 25).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Beartic",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageIs(50), ..CoinSpec::DEFAULT }))] },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_ATTACK)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
