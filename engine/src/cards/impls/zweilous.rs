//! Zweilous (SSP 118): Stomp Off: discard the top 2 cards of your opponent's deck. Darkness Fang: 60.
//!
//! A Discard event over the opponent's deck (this attack its cause).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zweilous",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
