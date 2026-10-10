//! Poké Pad (ASC / MC): search your deck for a Pokémon that doesn't have a
//! Rule Box, reveal it, put it into your hand, then shuffle.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokePad",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::RuleBox)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: true,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
