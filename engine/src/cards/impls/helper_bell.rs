//! Call Bell / Helper Bell (SSP): only if you go second, on your first turn:
//! search your deck for a Supporter card, reveal it, and put it into your
//! hand. Then, shuffle your deck.
//!
//! Twinleaf: playable only on game turn 2; the card moves supporter→discard
//! before the wait-less shuffle.
//!
//! Fixed (phase 4b, R2): the Supporter was never revealed; a ShowCardsPrompt
//! for the opponent now follows the move to the hand (SHOW_CARDS_TO_PLAYER).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HelperBell",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::Turn, CmpOp::Eq, Num::Lit(2))],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
