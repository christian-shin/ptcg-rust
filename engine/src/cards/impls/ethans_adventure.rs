//! Ethan's Adventure (DRI 165): search your deck for up to 3 in any combination of Ethan's Pokémon and Basic [R] Energy
//! cards, reveal them, and put them into your hand. Then, shuffle your deck.
//!
//! The chosen cards are revealed and go into the hand (PutIntoHand, this Supporter its cause); the deck is shuffled after
//! the search, even when nothing was taken (APR E-19).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "EthansAdventure",
    // Search your deck for up to 3 in any combination of Ethan's Pokémon and Basic [R] Energy,
    // reveal them, put them into your hand, shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::ETHANS)]), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fire Energy")])]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
