//! Team Rocket's Bother-Bot (DRI 172, Item): turn 1 of your opponent's
//! face-down Prize cards face up and choose a random card from your opponent's
//! hand. Your opponent reveals that card. You may have your opponent switch
//! those cards. (That Prize card remains face up for the rest of the game.)
//!
//! Playable unless the opponent has no face-down Prize card and no card in hand (with cards in hand and none face
//! down you only look at a random one, id2167). A face-down Prize card of the opponent's is chosen (face-up ones
//! are not offered) and stays face up; with an empty opposing hand it is only shown; otherwise a random card of their hand
//! is picked, both are shown and you may have them switch (the hand card goes to the Prize slot, still face up).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsBotherBotDRIPool",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        // Playable unless the opponent has no face-down Prize card and no card in hand.
        needs: &[Cond::Any(&[Cond::FaceDownPrize(Who::Opp), Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any)])],
        steps: &[Step::new(Op::BotherBot(BotherBotSpec { whose: Who::Opp }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
