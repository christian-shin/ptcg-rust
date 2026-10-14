//! Call Bell / Helper Bell (SSP 165): "You can use this card only if you go second, and only during your first turn." Search your
//! deck for a Supporter card, reveal it, and put it into your hand. Then, shuffle your deck.
//!
//! The first turn of the second player is game turn 2: the printed condition (`Cond::Printed`). One Search op revealing what it
//! finds, then one shuffle after it (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HelperBell",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Printed(&Cond::Cmp(Num::Turn, CmpOp::Eq, Num::Lit(2)))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
