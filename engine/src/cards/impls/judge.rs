//! Judge (FST, SVI as POR): each player shuffles their hand into their deck and draws 4
//! cards (the player first; the opponent's sequence starts after the
//! player's draw, as Twinleaf's `afterDraw` callback).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Judge@FST|POR",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Any(&[Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)])],
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(4) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(4) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
