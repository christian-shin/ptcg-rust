//! Centiskorch (PBL 10): Controlled Burn - discard the top 2 cards of your opponent's deck. Heat Tackle - 160; this Pokémon also does 30 damage to itself.
//!
//! A Discard of the top 2 cards of the opponent's deck (Deino SSP 117, Hydreigon ex); Heat Tackle's self damage follows Arrokuda DRI 62.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CentiskorchPBL",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(self_damage(30)),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
