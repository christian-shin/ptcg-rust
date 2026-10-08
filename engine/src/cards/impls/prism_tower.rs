//! Prism Tower (CRI / M4, stadium): once during each player's turn, that
//! player may discard 2 cards from their hand in order to draw a card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PrismTower",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Ge, Num::Lit(2)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(2), max: Num::Lit(2) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
