//! Mega Signal (MEG): search your deck for a Mega Evolution Pokémon ex,
//! reveal it, and put it into your hand. Then, shuffle your deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaSignal",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::POKEMON_EX_LOWER), Pred::Tag(crate::types::tag::POKEMON_SV_MEGA)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
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
