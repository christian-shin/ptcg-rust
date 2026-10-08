//! Iris's Fighting Spirit (JTG / ASC): you can use this card only if you
//! discard another card from your hand. Draw cards until you have 6 cards in
//! your hand.
//!
//! Fixed (phase 4b #39): `reduceEffect` throws SUPPORTER_ALREADY_PLAYED after
//! another Supporter and CANNOT_PLAY_THIS_CARD without another card in the
//! hand; the discard prompt is min 1, max 1, no cancel (it used to allow an
//! empty choice, which played the card for nothing). The Supporter has left
//! the hand by the time the prompt is answered, so only other cards are
//! listed. Then DRAW_CARDS_UNTIL_CARDS_IN_HAND (plain `deck.moveTo(hand, n)`)
//! unless the hand already has 6 or more cards.
//!
//! R7C: unplayable when it would draw nothing (empty deck, or 7 or more other cards in
//! the hand so that 6 are left after discarding one; rulings 851, 959, 1037).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IrisFightingSpirit",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Ge, Num::Lit(1)), Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Lt, Num::Lit(7)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Hand), to: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
