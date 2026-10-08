//! Ethan's Adventure (DRI): search your deck for up to 3 in any combination
//! of Ethan's Pokémon and Basic [R] Energy, reveal them, put them into your
//! hand, shuffle.
//!
//! Twinleaf: the card moves to the supporter pile, the empty-deck check
//! follows, non-matching cards are blocked (Energy must be a Basic card
//! named "Fire Energy"). The Supporter is discarded right after the prompt
//! is created, before it is answered; the callback (when something was
//! chosen) creates the ShowCards prompt, moves the cards and SHUFFLE_DECKs
//! (shuffle + silent wait).
//!
//! Fixed (phase 4b, R3): the deck is shuffled even when nothing was taken
//! (the callback used to return before SHUFFLE_DECK).
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
                msg: "CHOOSE_CARD_TO_DECK",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
