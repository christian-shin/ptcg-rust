//! Drakloak (TWM / ASC): Recon Directive — once during your turn, you may look at the top 2 cards of your deck and put 1 of
//! them into your hand. Put the other card on the bottom of your deck.
//!
//! The Look sets the cards aside (no zone change); the chosen one goes to the hand (a PutIntoHand of the Ability's effect)
//! and the other to the bottom (a PutIntoDeck). With 1 card left in the deck the Ability can be used and takes it (JP FAQ
//! Drakloak). The choice can't be cancelled.
use crate::spec::prelude::*;

const TOP: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(1));

pub static SPEC: CardSpec = CardSpec {
    class: "Drakloak",
    // Recon Directive: once during your turn, look at the top 2 cards of your deck and put 1 of
    // them into your hand; the other goes on the bottom of your deck.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), into: 1, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { from: TOP, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: TOP, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: TOP, cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
