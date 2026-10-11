//! Team Rocket's Archer (DRI 170): you can use this card only if any of your Team Rocket's Pokémon were Knocked Out during
//! your opponent's last turn. Each player shuffles their hand into their deck. Then, you draw 5 cards, and your opponent draws
//! 3 cards.
//!
//! The printed condition is `Cond::Printed` (it also holds when the effect is used by Look-Alike Show, id2225). Each player's hand goes
//! into their deck and is shuffled (PutIntoDeck, owner = that player), then they draw (Draw events): the opponent 3, then
//! you 5; playable with an empty deck (the hand is shuffled into it first).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsArcher",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::KnockedOutLastTurn { who: Who::Me, tag: Some(tag::TEAM_ROCKET), by_attack_damage: false })],
        // Each player shuffles their hand into their deck; you draw 5 cards and your opponent draws 3.
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(3) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(5) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
