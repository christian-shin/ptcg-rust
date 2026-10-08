//! Great Tusk (TEF): Land Collapse — discard the top card of your opponent's
//! deck; if you played an Ancient Supporter from your hand this turn,
//! discard 3 more. Giant Tusk — 160.
//!
//! Twinleaf reads `player.ancientSupporter` (set by Explorer's Guidance /
//! Professor Sada's Vitality, cleared at the end of the turn).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GreatTusk",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(1)), ..MoveSpec::DEFAULT })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::AncientSupporterPlayed(Who::Me),
                yes: &[Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(3)), ..MoveSpec::DEFAULT }))],
                no: &[],
            }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
