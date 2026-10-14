//! Brock's Scouting (JTG): search your deck for up to 2 Basic Pokémon or 1 Evolution Pokémon, reveal them, put them into
//! your hand. Then, shuffle your deck.
//!
//! A Supporter: the Search puts the cards into the hand (a PutIntoHand of the Supporter's effect, revealed) and the shuffle
//! follows it (APR E-19). Searching a deck known to be empty is impossible, so the card can't be played with an empty deck
//! (id840, id925).
use crate::spec::prelude::*;

const EVOLUTION: Pred = Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)]);
const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "BrocksScouting",
    // Search your deck for up to 2 Basic Pokémon or 1 Evolution Pokémon, reveal them, put them
    // into your hand, then shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::Pokemon,
                    bounds: Bounds {
                        min: Num::Lit(0),
                        max: Num::Add(&Num::Min(&Num::CardCount(DECK, Pred::Basic), &Num::Lit(2)), &Num::Min(&Num::CardCount(DECK, EVOLUTION), &Num::Lit(1))),
                    },
                    caps: &[Cap { kind: CapKind::Basic, max: Num::Min(&Num::CardCount(DECK, Pred::Basic), &Num::Lit(2)) }, Cap { kind: CapKind::Evolution, max: Num::Min(&Num::CardCount(DECK, EVOLUTION), &Num::Lit(1)) }],
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
