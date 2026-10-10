//! Lt. Surge's Bargain (MEG 120, Supporter): ask your opponent if each player
//! may take a Prize card. If yes, each player takes a Prize card. If no, you
//! draw 4 cards.
//!
//! Your opponent answers the confirm. Yes: you take a Prize card, then your opponent does (two TakePrizes events of
//! the Supporter's effect; the second is skipped when you have no Prize cards left). No: you draw 4.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LtSurgesBargainMEGPool",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::May(MaySpec { asker: Who::Opp, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Gt, Num::Lit(0)), yes: &[Step::new(Op::TakePrize(TakePrizeSpec { who: Who::Me, count: Num::Lit(1) })), Step::new(Op::TakePrize(TakePrizeSpec { who: Who::Opp, count: Num::Lit(1) }))], no: &[] }))], no: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(4)) }))] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
