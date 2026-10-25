//! Krookodile (BLK 59): Tighten Up - 60; your opponent discards 2 cards from their hand. Cursed Slug - 120+; if your opponent has 3
//! or fewer cards in their hand, this attack does 120 more damage. This is a different card from Krookodile PFL 66.
//!
//! Tighten Up is Krokorok BLK 58's (the opponent chooses 2 cards of their hand and discards them). Cursed Slug compares the
//! opponent's hand size before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KrookodileBLK",
    attacks: &[
        AttackSpec { index: 0, steps: &super::krokorok_blk::TIGHTEN_UP },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(120, Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), CmpOp::Le, Num::Lit(3))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
