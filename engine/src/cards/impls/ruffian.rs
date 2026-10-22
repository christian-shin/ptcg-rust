//! Ruffian (JTG 157): Supporter. Discard a Pokémon Tool and a Special Energy from 1 of your opponent's Pokémon.
//!
//! One target Pokémon is chosen (one that has a Tool or a Special Energy card); each card discarded is a LeavePlay (D1)
//! caused by this Supporter, to its owner's discard pile: one Tool (chosen when there are several), then one Special
//! Energy card. The part that can be done is done (APR II.A: do as much as you can).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ruffian",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::OneOf(&[SlotPred::AnyTool(Pred::Any), SlotPred::HasCard(Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]))])) }, purpose: PromptPurpose::DiscardCard })),
            // One Tool (asked when there are several), then one Special Energy.
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::ToolCount(SlotSel::One(SlotExpr::Picked)), CmpOp::Gt, Num::Lit(1)),
                yes: &[
                    Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), predicate: Pred::Tool, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
                    Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                ],
                no: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(SlotExpr::Picked)), ..DiscardSpec::DEFAULT }))],
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)])), CmpOp::Gt, Num::Lit(0)),
                yes: &[
                    Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), predicate: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })),
                    Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), cards: CardSel::Chosen(1), ..DiscardSpec::DEFAULT })),
                ],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
