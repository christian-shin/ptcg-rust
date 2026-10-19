//! Gumshoos (M1L): Evidence Gathering - once during your turn, you may use this Ability. Switch a card from your hand with the
//! top card of your deck. Bite - 50.
//!
//! The switch is simultaneous (id2209: you don't look at the top card first): the top card goes to the hand (PutIntoHand), the
//! chosen card goes on top of the deck (PutIntoDeck, position Top). Usable with a card in hand and in the deck; once used,
//! the switch is made (the card from the hand is chosen with no cancel). Once per turn by its usage record, made when it is
//! used (id127, id249) and ending with the turn.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Gumshoos",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), position: DeckPosition::Top, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
