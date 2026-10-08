//! Unfair Stamp (TWM, ACE SPEC): only if one of your Pokémon was Knocked Out
//! during your opponent's last turn. Each player shuffles their hand into
//! their deck; you draw 5 cards and your opponent draws 2.
//!
//! Every Unfair Stamp copy (in any zone) adds its own marker to its owner when
//! that player's Pokémon is Knocked Out during the opponent's turn. Phase 4b:
//! playable with an empty deck (the hand is shuffled into it first).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "UnfairStamp",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::KnockedOutLastTurn { who: Who::Me, tag: None, by_attack: false }],
        // Each player shuffles their hand into their deck; you draw 5 cards and your opponent draws 2.
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(5) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(2) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
