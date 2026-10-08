//! Gurdurr (BLK 48): Low Kick — 30. Hammer Arm — 60; discard the top card of
//! your opponent's deck (DISCARD_TOP_X_OF_OPPONENTS_DECK).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GurdurrBLKPool",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(1)), ..MoveSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
