//! Hand Trimmer (TEF): both players discard cards from their hand until they
//! each have 5 cards in hand (opponent first).
//!
//! Twinleaf: both ChooseCardsPrompts are queued at once (opponent's first),
//! each only when that hand has more than 5 cards; each callback discards
//! the chosen cards.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HandTrimmer",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Any(&[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Gt, Num::Lit(5)), Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Gt, Num::Lit(5))])],
        steps: &[
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Gt, Num::Lit(5)), yes: &[Step::new(Op::Pick(PickSpec { chooser: Who::Opp, from: ZoneRef(Who::Opp, Zone::Hand), bounds: Bounds { min: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Opp, Zone::Hand), Pred::Any), &Num::Lit(5))), max: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Opp, Zone::Hand), Pred::Any), &Num::Lit(5))) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Gt, Num::Lit(5)), yes: &[Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), &Num::Lit(5))), max: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), &Num::Lit(5))) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
