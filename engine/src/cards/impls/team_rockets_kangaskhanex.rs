//! Team Rocket's Kangaskhan ex (ASC): Comet Punch — flip 4 coins, 30 damage
//! for each heads. Wicked Impact — 120+, 100 more if you played a Team
//! Rocket Supporter from your hand this turn (`player.rocketSupporter`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsKangaskhanex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::Coin(CoinSpec { mode: CoinMode::Fixed(4), then: &[Step::new(damage_is(Num::Mul(&Num::Heads, &Num::Lit(30))))], ..CoinSpec::DEFAULT }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(100, Cond::RocketSupporterPlayed(Who::Me)))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
