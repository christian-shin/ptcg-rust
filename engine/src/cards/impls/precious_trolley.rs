//! Precious Trolley (SSP 185, ACE SPEC): search your deck for any number of Basic Pokémon and put them onto your Bench. Then,
//! shuffle your deck.
//!
//! One Search op onto the Bench, 0 to the open spots (each Pokémon an EnterPlay), then one shuffle after it (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PreciousTrolley",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Basic]), bounds: Bounds { min: Num::Lit(0), max: Num::OpenBench(Who::Me) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                msg: "",
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
