//! Roto-Stick (PRE 127, Item): look at the top 4 cards of your deck. You may reveal any number of Supporter cards you find there
//! and put them into your hand. Shuffle the other cards back into your deck.
//!
//! Look sets the 4 cards aside (a register, nothing reacts to it, D2); the Supporters chosen are revealed and put into your hand
//! (PutIntoHand), the others go back into the deck and the deck is shuffled, also when every card was taken (id2377).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RotoStick",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(4)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(4) }, into: 1, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
