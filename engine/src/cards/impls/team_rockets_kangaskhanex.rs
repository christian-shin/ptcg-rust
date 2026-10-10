//! Team Rocket's Kangaskhan ex (ASC): Comet Punch — flip 4 coins, 30 damage
//! for each heads. Wicked Impact — 120+, 100 more if you played a Team
//! Rocket Supporter from your hand this turn (`player.rocketSupporter`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsKangaskhanex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::Count(4), per_heads: PerHeads::DamageIs(30), ..CoinSpec::DEFAULT }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(100, Cond::PlayedThisTurn(Who::Me, Pred::All(&[Pred::Supporter, Pred::Tag(crate::types::tag::TEAM_ROCKET)]))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
