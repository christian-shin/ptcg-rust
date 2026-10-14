//! Gumshoos (M1L): Evidence Gathering - once during your turn, you may use this Ability. Switch a card from your hand with the
//! top card of your deck. Bite - 50.
//!
//! The switch is simultaneous (id2209: you don't look at the top card first): the top card goes to the hand (PutIntoHand), the
//! chosen card goes on top of the deck (PutIntoDeck, position Top). Usable with a card in hand and in the deck; once per turn
//! by its usage record, which ends with the turn.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Gumshoos",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Copy))), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, into: 0, ..PickSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..PutIntoHandSpec::DEFAULT })), Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), position: DeckPosition::Top, ..PutIntoDeckSpec::DEFAULT })), Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Copy)))], no: &[] })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
