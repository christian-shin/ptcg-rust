//! Deino (SSP 117): Stomp Off: discard the top card of your opponent's deck.
//!
//! A Discard event over the opponent's deck (this attack its cause); no look, no shuffle (nothing in the text).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Deino",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..DiscardSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
