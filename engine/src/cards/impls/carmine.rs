//! Carmine (TWM / PRE): if you go first, you can use this card on your first turn. Discard your hand and draw 5 cards.
//!
//! Discarding the hand is a cost, not the effect (id1124, APR E-30): `Step::cost`. With an empty deck nothing could be drawn,
//! so the card can't be played (id925, id2231); with an empty hand and cards in the deck it can (id1125; JP FAQ Carmine
//! with only Carmine in the hand). The Draw is one Draw event of the Supporter's effect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Carmine",
    // Discard your hand and draw 5 cards (discarding the hand is a cost: nothing can be drawn from
    // an empty deck, id1124).
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            // An empty hand with a non-empty deck is playable (id1125).
            Step::new(Op::If(IfSpec {
                cond: Cond::NonemptyOther(ZoneRef(Who::Me, Zone::Hand), Pred::Any),
                yes: &[Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, ..DiscardSpec::DEFAULT }))],
                no: &[],
            })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(5)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
