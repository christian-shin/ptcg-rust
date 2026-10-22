//! Liepard (WHT 56): Knock Off - 50; discard a random card from your opponent's hand.
//!
//! One Discard of 1 random card from the opponent's hand, an effect of the attack after the damage (Scraggy ASC 134).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Liepard",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
