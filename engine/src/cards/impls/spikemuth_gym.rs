//! Spikemuth Gym (DRI, Stadium): once during each player's turn, that player
//! may search their deck for a Marnie's Pokémon, reveal it, put it into their
//! hand, then shuffle.
//!
//! Twinleaf: the search callback creates the ShowCards prompt (if a card was
//! chosen), moves the card and creates the shuffle prompt without waiting;
//! the shuffle has no animation wait.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "SpikemuthGym",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(tag::MARNIES)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
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
