//! Zweilous (SSP): Stomp Off — discard the top 2 cards of your opponent's deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zweilous",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(2)), ..MoveSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
