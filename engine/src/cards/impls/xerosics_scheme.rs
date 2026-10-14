//! Xerosic's Machinations (SFA 64): your opponent discards cards from their hand until they have 3 cards in their hand.
//!
//! The opponent chooses what to discard (a pick of as many as they hold beyond 3), then one Discard from their hand (D3). Usable
//! while their hand holds more than 3.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "XerosicsScheme",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Gt, Num::Lit(3))],
        steps: &[
            // Your opponent discards cards from their hand until they have 3 cards in their hand.
            Step::new(Op::Pick(PickSpec {
                chooser: Who::Opp,
                from: ZoneRef(Who::Opp, Zone::Hand),
                bounds: Bounds { min: Num::Sub(&Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), &Num::Lit(3)), max: Num::Sub(&Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), &Num::Lit(3)) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
