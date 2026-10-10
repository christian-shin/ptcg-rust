//! Team Rocket's Chingling (DRI): Ring Ring Noise — discard a random card
//! from your opponent's hand.
//!
//! Twinleaf: on AFTER_ATTACK, `Chance.index(hand.length)` picks the card and
//! MOVE_CARDS discards it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsChingling",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), ..DiscardSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
