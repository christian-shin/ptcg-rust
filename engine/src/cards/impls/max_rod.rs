//! Max Rod (PRE, ACE SPEC): choose up to 5 in any combination of Pokémon and
//! Basic Energy cards from your discard pile and put them into your hand.
//!
//! Twinleaf: other cards are blocked; the choice is min 1 / max 5 with no
//! cancel on the (sorted) discard; the reveal comes before MOVE_CARDS.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MaxRod",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::OneOf(&[Pred::Pokemon, Pred::BasicEnergy]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(5) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
