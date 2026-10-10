//! Miraculous Intercom (SSP, ACE SPEC Item): put up to 2 Supporter cards
//! from your discard pile into your hand.
//!
//! Twinleaf quirks kept: the prompt requires at least 1 card; the card is
//! MOVE_CARDS'd hand -> supporter pile (already there), then after the
//! reveal hand -> discard (a no-op), the chosen cards discard -> hand, and
//! finally supporter pile -> discard.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiraculousIntercom",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
