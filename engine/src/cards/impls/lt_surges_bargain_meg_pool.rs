//! Lt. Surge's Bargain (MEG 120, Supporter): ask your opponent if each player
//! may take a Prize card. If yes, each player takes a Prize card. If no, you
//! draw 4 cards.
//!
//! Twinleaf: throws SUPPORTER_ALREADY_PLAYED when `supporterTurn > 0`; the
//! OPPONENT answers a ConfirmPrompt. Yes: TAKE_X_PRIZES(player, 1) with a
//! callback that runs TAKE_X_PRIZES(opponent, 1) (the callback is skipped when
//! the player has no Prize cards left). No: DRAW_CARDS(player, 4).
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
