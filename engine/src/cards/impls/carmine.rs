//! Carmine (TWM): if you go first, you can use this card on your first turn.
//! Discard your hand and draw 5 cards.
//!
//! Twinleaf: throws when a Supporter was already played, or when the deck is
//! empty (rulings 1037/1038: discarding the hand is a cost, not the effect; R5
//! had made it playable with an empty deck, phase 4b reverted that); the other hand
//! cards go to the discard pile in one MOVE_CARDS (no source card, only when
//! there are any), then DRAW_CARDS 5.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Carmine",
    // Discard your hand and draw 5 cards (discarding the hand is a cost: nothing can be drawn from
    // an empty deck, ruling 1037).
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            // An empty hand with a non-empty deck is playable (ruling 1038).
            Step::new(Op::If(IfSpec {
                cond: Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any),
                yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, ..DiscardSpec::DEFAULT }))],
                no: &[],
            })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(5)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
