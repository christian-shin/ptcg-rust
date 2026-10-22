//! Scraggy (ASC 134): Knock Off - 20; discard a random card from your opponent's hand.
//!
//! One Discard from the opponent's hand, 1 random card, an effect of the attack after the damage (Team Rocket's Chingling
//! DRI).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScraggyASC",
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
