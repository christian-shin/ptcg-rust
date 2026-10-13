//! Cyrano (SSP 170): search your deck for up to 3 Pokémon ex, reveal them, and put them into your hand. Then, shuffle your
//! deck.
//!
//! One Search op revealing what it finds, then one shuffle after it, also when nothing was found (APR E-19, id2303). The
//! Supporter's own play (one per turn, the discard afterward) is PlayTrainer, not this card's business.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cyrano",
    // Search your deck for up to 3 Pokémon ex, reveal them, put them into your hand, then shuffle.
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec {
                    predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonEx]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
