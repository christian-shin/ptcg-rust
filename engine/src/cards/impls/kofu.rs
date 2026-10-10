//! Kofu (SCR, supporter): put 2 cards from your hand on the bottom of your
//! deck in any order, then draw 4 cards.
//!
//! Twinleaf: the hand cards are chosen from `player.hand` (2 required, no
//! cancel); the core TrainerEffect reducer has already moved the Supporter to
//! the supporter pile when the prompt is answered, so Kofu itself is never a
//! pick (phase 4b #43: checked in corpus traces, not a bug). After the
//! order prompt the cards go to the deck bottom and `min(4, deck size)` cards
//! are moved to the hand (no shuffle, no supporter-turn marker).
//!
//! R7C: counts the cards other than Kofu in the hand (as the effect of an attack Kofu is
//! not in the hand).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Kofu",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Ge, Num::Lit(2))],
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
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Min(&Num::Lit(4), &Num::ZoneSize(ZoneRef(Who::Me, Zone::Deck)))) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
