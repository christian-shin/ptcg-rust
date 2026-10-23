//! Billy & O'Nare (JTG 142, Supporter): draw 2 cards. Then, if you have 10 or more cards in your hand, draw 2 more cards.
//!
//! Two Draw events of this Supporter. The count of the second is read when it is reached, after the first draw, without the
//! Supporter itself (it is in the play area, not in the hand, APR B-01; `Num::OthersCount`: the program runs while the card
//! still sits in the hand). With an empty deck the card can't be played (a Trainer that can
//! change nothing: id925, id2231).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BillyONare",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Ge, Num::Lit(10)),
                yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
