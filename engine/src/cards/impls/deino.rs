//! Deino (SSP): Stomp Off — discard the top card of your opponent's deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Deino",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(1)), ..MoveSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
