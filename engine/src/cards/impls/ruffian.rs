//! Ruffian (JTG, supporter): discard a Pokémon Tool and a Special Energy
//! from 1 of your opponent's Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Ruffian",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::OneOf(&[SlotPred::AnyTool(Pred::Any), SlotPred::HasCard(Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]))]))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::OneOf(&[SlotPred::AnyTool(Pred::Any), SlotPred::HasCard(Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]))])),
                msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
            })),
            // One Tool (asked when there are several), then one Special Energy.
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::ToolCount(SlotExpr::Picked), CmpOp::Gt, Num::Lit(1)),
                yes: &[
                    Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), predicate: Pred::Tool, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
                    Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
                ],
                no: &[Step::new(Op::Move(MoveSpec { to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Tools(SlotExpr::Picked), ..MoveSpec::DEFAULT }))],
            })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)])), CmpOp::Gt, Num::Lit(0)),
                yes: &[
                    Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), predicate: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 1, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
                    Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(1), ..MoveSpec::DEFAULT })),
                ],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
