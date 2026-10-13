//! Team Rocket's Kangaskhan ex (ASC 162): Comet Punch — flip 4 coins, 30 damage for each heads. Wicked Impact — 120+; if you played a
//! Supporter card that has "Team Rocket" in its name from your hand during this turn, 100 more damage.
//!
//! Wicked Impact reads the turn's play record (`Cond::PlayedThisTurn`): a Supporter used through Look-Alike Show isn't played from the
//! hand (id2225).
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
