//! Hole-Digging Shovel (POR 74, Item): discard the top 2 cards of your deck.
//!
//! One Discard event of the top 2 cards of the deck (all of it when it holds fewer). With an empty deck the card can't be
//! played (a Trainer that can change nothing: id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HoleDiggingShovel",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), into: None }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
