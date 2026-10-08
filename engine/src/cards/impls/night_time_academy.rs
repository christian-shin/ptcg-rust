//! Academy at Night (SFA): once during each player's turn, that player may
//! put a card from their hand on top of their deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NightTimeAcademy",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DECK", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), place: Place::Top, ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
