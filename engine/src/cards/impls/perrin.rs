//! Perrin (TWM 160, Supporter): reveal up to 2 Pokémon in your hand and put them into your deck. If you do, search your deck
//! for up to that many Pokémon, reveal them, and put them into your hand. Then, shuffle your deck.
//!
//! The Pokémon are chosen from the hand (0 to 2), revealed, and put into the deck by one PutIntoDeck. "If you do" is
//! `Cond::Done` on it (APR E-20); the search then takes up to as many Pokémon as were put (the count of the register), revealing
//! them, and the shuffle follows (APR E-19). With no Pokémon in the hand nothing can be put and the card can't be played (id925).
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Perrin",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: HAND, predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), by: Who::Me, to: Who::Opp, when_empty: false })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: HAND, cards: CardSel::Chosen(0), position: DeckPosition::Top, order: DeckOrder::AsIs, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Done,
                yes: &[Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: DECK, predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::RegCount(0) }, into: 1, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: true },
                    cancel: false,
                }))],
                no: &[],
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
