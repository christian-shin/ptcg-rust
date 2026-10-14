//! Hand Trimmer (TEF 150): Item. Each player discards cards from their hand until they have 5 cards in their hand. Your
//! opponent discards first. (A player with 5 or fewer cards doesn't discard.)
//!
//! Each player with more than 5 cards (not counting this card for you) chooses the cards to discard and discards them
//! by one Discard event from their hand (cause: this Item), the opponent first. The `needs` entry (some hand over 5)
//! restates that a Trainer that can't have any effect can't be played (id925, id1042) and goes at the D10 switch.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HandTrimmer",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Any(&[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Gt, Num::Lit(5)), Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Gt, Num::Lit(5))])],
        steps: &[
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Gt, Num::Lit(5)), yes: &[Step::new(Op::Pick(PickSpec { chooser: Who::Opp, from: ZoneRef(Who::Opp, Zone::Hand), bounds: Bounds { min: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Opp, Zone::Hand), Pred::Any), &Num::Lit(5))), max: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Opp, Zone::Hand), Pred::Any), &Num::Lit(5))) }, into: 0, ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT }))], no: &[] })),
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Gt, Num::Lit(5)), yes: &[Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), &Num::Lit(5))), max: Num::Max(&Num::Lit(0), &Num::Sub(&Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), &Num::Lit(5))) }, into: 0, ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
