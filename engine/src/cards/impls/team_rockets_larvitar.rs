//! Team Rocket's Larvitar (DRI 94): Mountain Munch - 10; discard the top card of your opponent's deck.
//!
//! A Discard event over the opponent's deck (this attack its cause); no look, no shuffle (Deino SSP 117).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsLarvitar",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
