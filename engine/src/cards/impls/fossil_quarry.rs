//! Fossil Quarry (PBL 76, Stadium): once during each player's turn, that player may search their deck for up to 2 Item cards that
//! have "Antique" in their name and put them onto their Bench. Then, that player shuffles their deck.
//!
//! A Stadium use (UseStadium by either player, once per turn each): one search of 0 to 2 Item cards whose name contains "Antique"
//! (the Fossils), put onto the user's Bench by the effect (EnterPlay by an effect, from the deck: id2233; a Fossil in play is a
//! Pokémon, ruling 752, and not played from the hand, so a lock over playing Items from the hand doesn't stop it: id2325), at
//! most as many as the Bench has room for, then the shuffle (APR E-19). It can't be used with no such card in the deck or no room
//! on the Bench (a Stadium use that can change nothing isn't allowed, id925).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "FossilQuarry",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: DECK, predicate: Pred::All(&[Pred::Item, Pred::NameContains("Antique")]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
