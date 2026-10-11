//! Megaton Blower (SSP 182, ACE SPEC): discard all Pokémon Tools and Special Energy from all of your opponent's Pokémon,
//! and discard a Stadium in play.
//!
//! The Stadium, then each of the opponent's Pokémon's Special Energy and Tools, leave play (LeavePlay events, this Item
//! their cause); a protection from this Trainer's effect is a Prevent read by each event. Usable when there is a Stadium or
//! an opponent's Pokémon has a Tool or Special Energy (usability by the rules, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegatonBlower",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Any(&[Cond::StadiumInPlay(Pred::Any), Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::OneOf(&[SlotPred::AnyTool(Pred::Any), SlotPred::HasCard(Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]))]))])],
        steps: &[
            Step::new(DISCARD_STADIUM),
            Step::new(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Opp),
                body: &[Step::new(Op::If(IfSpec {
                    // A protection from this Trainer's effect is a Prevent read by each Discard / LeavePlay (batch 6/7).
                    cond: Cond::True,
                    yes: &[
                        Step::new(Op::Snapshot(SnapshotSpec { zone: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), predicate: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]), into: 0 })),
                        Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                        Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Tools(SlotExpr::Picked)), ..DiscardSpec::DEFAULT })),
                    ],
                    no: &[],
                }))],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
