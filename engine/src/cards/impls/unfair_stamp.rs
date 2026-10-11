//! Unfair Stamp (TWM 165, ACE SPEC): you can use this card only if one of your Pokémon was Knocked Out during your opponent's
//! last turn. Each player shuffles their hand into their deck. Then, you draw 5 cards, and your opponent draws 2 cards.
//!
//! The printed condition is `Cond::Printed`. Each player's hand goes into their deck and is shuffled (PutIntoDeck, owner =
//! that player), then they draw (Draw events): you 5, then the opponent 2; playable with an empty deck (the hand is shuffled
//! into it first).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "UnfairStamp",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Printed(&Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: false, tag: None })],
        // Each player shuffles their hand into their deck; you draw 5 cards and your opponent draws 2.
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(5) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(2) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
