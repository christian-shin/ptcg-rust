//! Emcee's Hype (DRI 163, Supporter): draw 2 cards. If your opponent has 3 or fewer Prize cards remaining, draw 2 more cards.
//!
//! Two Draw events of this Supporter; the condition reads the opponent's Prize cards left when it is reached. With an empty
//! deck the card can't be played (id925, id2231).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EmceesHype",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(3)),
                yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
