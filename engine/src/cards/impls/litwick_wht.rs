//! Litwick (WHT 16): Brighten and Burn — look at the top card of your deck. You may discard that card.
//!
//! The card is looked at (set aside); discarded if you choose to, else it goes back on top of the deck (id2189).
use crate::spec::prelude::*;

const LOOKED: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(0));

pub static SPEC: CardSpec = CardSpec {
    class: "LitwickWHT",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), into: 0, ..LookSpec::DEFAULT })),
            Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[Step::new(Op::Discard(DiscardSpec { from: LOOKED, ..DiscardSpec::DEFAULT }))],
                no: &[Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: LOOKED, cards: CardSel::All, position: DeckPosition::Top, ..PutIntoDeckSpec::DEFAULT }))],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
