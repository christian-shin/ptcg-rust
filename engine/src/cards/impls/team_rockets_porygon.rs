//! Team Rocket's Porygon (DRI): Hacking — discard a card from your hand. If
//! you do, your opponent discards a card from their hand.
//!
//! Twinleaf: nothing happens with an empty hand; the opponent chooses their
//! own card (only if their hand is non-empty).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPorygon",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::after_damage(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Pick(PickSpec { chooser: Who::Opp, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })), Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Hand), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(1), ..MoveSpec::DEFAULT }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
