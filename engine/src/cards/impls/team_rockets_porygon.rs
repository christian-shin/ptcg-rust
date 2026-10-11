//! Team Rocket's Porygon (DRI): Hacking — discard a card from your hand. If you do, your opponent discards a card from their
//! hand.
//!
//! Not a cost: the attack's effect. Your Discard comes first (a Discard from the hand, the card chosen by you); "if you do" is
//! `Cond::Done` on it (APR E-20), then the opponent chooses their own card and discards it. An empty hand does nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPorygon",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::after_damage(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Pick(PickSpec { chooser: Who::Opp, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 1, ..PickSpec::DEFAULT })), Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(1), ..DiscardSpec::DEFAULT }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
