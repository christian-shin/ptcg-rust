//! Furfrou (M3): Hand Trim — discard random cards from your opponent's hand
//! until they have 5 cards in their hand. Headbutt — 30.
//!
//! Twinleaf draws each discard with `Chance.index(hand.length)`.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Furfrou",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Max(&Num::Lit(0), &Num::Sub(&Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), &Num::Lit(5)))), ..DiscardSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
