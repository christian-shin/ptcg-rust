//! Spikemuth Gym (DRI, Stadium): once during each player's turn, that player may search their deck for a Marnie's Pokémon,
//! reveal it, and put it into their hand. Then, that player shuffles their deck.
//!
//! A Stadium use (UseStadium): the Search puts the card into the hand revealed (a PutIntoHand of the Stadium's effect, none
//! required: a search for a kind); the shuffle follows (APR E-19). It needs a card in the deck.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "SpikemuthGym",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(tag::MARNIES)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
