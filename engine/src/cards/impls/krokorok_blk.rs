//! Krokorok (BLK 58): Tighten Up - 40; your opponent discards 2 cards from their hand. This is a different card from Krokorok PFL 65.
//!
//! After the damage the opponent chooses 2 cards of their hand (all of it when fewer) and discards them (a Discard caused by the
//! attack); nothing happens with an empty hand. The steps are shared with the Krookodile cards that print the same text.
use crate::spec::prelude::*;

const OPP_HAND: ZoneRef = ZoneRef(Who::Opp, Zone::Hand);

pub static TIGHTEN_UP: [Step; 2] = [
    Step::after_damage(Op::Pick(PickSpec {
        chooser: Who::Opp,
        from: OPP_HAND,
        bounds: Bounds { min: Num::Min(&Num::ZoneSize(OPP_HAND), &Num::Lit(2)), max: Num::Min(&Num::ZoneSize(OPP_HAND), &Num::Lit(2)) },
        into: 0,
        ..PickSpec::DEFAULT
    })),
    Step::after_damage(Op::Discard(DiscardSpec { from: OPP_HAND, cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
];

pub static SPEC: CardSpec = CardSpec { class: "KrokorokBLK", attacks: &[AttackSpec { index: 0, steps: &TIGHTEN_UP }], ..CardSpec::NONE };

pub static IMPL: CardImpl = SPEC.card_impl();
