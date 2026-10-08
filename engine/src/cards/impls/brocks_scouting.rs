//! Brock's Scouting (JTG): search your deck for up to 2 Basic Pokémon or 1
//! Evolution Pokémon, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf order kept: the card moves itself to the supporter pile; the
//! final ShuffleDeckPrompt has no trailing WaitPrompt. Fixed (R1-16, rulings
//! 779 and 851): it can't be played with an empty deck (CANNOT_PLAY_THIS_CARD,
//! before the card moves).
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
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: DECK })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
