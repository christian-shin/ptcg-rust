//! Tatsugiri (TWM 131): Attract Customers - once during your turn, if this Pokémon is in the Active Spot, you may look at
//! the top 6 cards of your deck, reveal a Supporter card you find there, and put it into your hand. Shuffle the other cards
//! back into your deck. Surf - 50.
//!
//! The 6 cards are set aside (`Op::Look`); the Supporter taken (none is allowed: "you may") is revealed and put into the
//! hand by one PutIntoHand event (cause: this Ability); the others go back into the deck (PutIntoDeck) and the deck is
//! shuffled once.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tatsugiri",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::IsActive(SlotExpr::This)],
        // Look at the top 6 cards of your deck, reveal a Supporter there and put it into your hand; shuffle the rest back.
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Scratch(0)),
                predicate: Pred::Supporter,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                into: 1,
                purpose: Some(PromptPurpose::PutIntoHand),
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Chosen(1), reveal: Some(Who::Opp), ..PutIntoHandSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
