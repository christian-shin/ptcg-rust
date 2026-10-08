//! Redeemable Ticket (JTG): count your Prize cards and shuffle them face
//! down, then put them at the bottom of your deck. If you do, add that many
//! cards from the top of your deck to your Prize cards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RedeemableTicket",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Gt, Num::Lit(0))],
        steps: &[Step::new(Op::PrizeVisibility(PrizeVisibilitySpec { action: PrizeAction::RedealThroughDeck }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
