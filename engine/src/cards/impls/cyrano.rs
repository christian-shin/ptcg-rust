//! Cyrano (SSP): search your deck for up to 3 Pokémon ex, reveal them, put
//! them into your hand, then shuffle.
//!
//! Twinleaf quirks kept: no supporter-turn check, the card stays in hand
//! until the search resolves (then it is moved hand → discard), and the
//! shuffle is a bare ShuffleDeckPrompt (no trailing wait).
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
                    predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::POKEMON_EX_LOWER)]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                    ..PickSpec::DEFAULT
                },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
