//! Kofu (SCR, Supporter): put 2 cards from your hand on the bottom of your deck in any order. If you put 2 cards on the
//! bottom of your deck in this way, draw 4 cards. (If you can't put 2 cards from your hand on the bottom of your deck, you
//! can't use this card.)
//!
//! The parenthesis is a printed condition (`Cond::Printed`): 2 cards other than Kofu in the hand (used by an attack, Kofu is
//! not in the hand, JP FAQ Kofu: it can be used with an empty deck). The 2 cards leave the hand as one PutIntoDeck (order
//! chosen by the player) and "If you put 2 cards ... in this way" is `Cond::DidAll` on it (all of what it says: a card of the two refused would make
//! it `Partial`, which `Cond::Done` would accept; APR E-20 / E-33), then one Draw event.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Kofu",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        // Printed: "(If you can't put 2 cards from your hand on the bottom of your deck, you can't use this card.)"
        needs: &[Cond::Printed(&Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Ge, Num::Lit(2)))],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(2), max: Num::Lit(2) }, into: 0, msg: "CHOOSE_CARDS_TO_PUT_ON_BOTTOM_OF_THE_DECK", ..PickSpec::DEFAULT })),
            // "Put 2 cards from your hand on the bottom of your deck in any order": set aside in register 1 while the order
            // is chosen.
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec {
                from: ZoneRef(Who::Me, Zone::Hand),
                cards: CardSel::Chosen(0),
                position: DeckPosition::Bottom,
                order: DeckOrder::ChosenBy(Who::Me),
                into: Some(1),
                ..PutIntoDeckSpec::DEFAULT
            })),
            // "If you put 2 cards on the bottom of your deck in this way, draw 4 cards."
            Step::new(Op::If(IfSpec { cond: Cond::DidAll, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Min(&Num::Lit(4), &Num::ZoneSize(ZoneRef(Who::Me, Zone::Deck)))) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
