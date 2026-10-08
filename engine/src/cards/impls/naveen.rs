//! Naveen (POR, supporter): discard any number of cards from your hand, then
//! draw cards until you have 5 cards in your hand.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Naveen",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                // A hand of 5 or more must discard enough to draw at least one card.
                bounds: Bounds { min: Num::Max(&Num::Lit(0), &Num::Sub(&Num::HandOthers(Who::Me), &Num::Lit(4))), max: Num::HandOthers(Who::Me) },
                into: 0,
                soft: true,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSizeOthers(Num::Lit(5)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
