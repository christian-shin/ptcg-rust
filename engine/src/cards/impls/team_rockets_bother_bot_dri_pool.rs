//! Team Rocket's Bother-Bot (DRI 172, Item): turn 1 of your opponent's
//! face-down Prize cards face up and choose a random card from your opponent's
//! hand. Your opponent reveals that card. You may have your opponent switch
//! those cards. (That Prize card remains face up for the rest of the game.)
//!
//! Twinleaf: throws CANNOT_PLAY_THIS_CARD when every non-empty Prize card of
//! the opponent is already face up and the opponent has no card in hand (phase
//! 4b, R2: with cards in hand it is playable and you only look at a random
//! one: a ShowCardsPrompt, then the card is discarded; ruling 1681); the card goes to the supporter zone by
//! hand (preventDefault). A ChoosePrizePrompt (opponent's Prizes, face-down
//! only, face-up ones blocked) picks the Prize; its list gets `faceUpPrize`.
//! An empty opposing hand: ShowCardsPrompt of the Prize card, then the card is
//! discarded. Otherwise `Chance.index(hand.length)` picks the hand card, a
//! ShowCardsPrompt shows Prize + hand card, a ConfirmPrompt asks the player;
//! on yes the Prize card goes to the hand and the hand card into the Prize
//! list (still face up). Finally the item moves supporter -> discard.
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
