//! Dialga (SSP 135): Time Manipulation - search your deck for 2 cards, shuffle your deck, then put those cards on top of it in
//! any order. Buster Tail - 160.
//!
//! The 2 cards are chosen (all of the deck when it holds fewer), set aside while the deck is shuffled (APR E-19), then put on
//! top by one PutIntoDeck event, ordered by the player (`DeckOrder::ChosenBy`), as Ciphermaniac's Codebreaking does.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "DialgaSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Deck),
                bounds: Bounds { min: Num::Min(&Num::Lit(2), &Num::ZoneSize(ZoneRef(Who::Me, Zone::Deck))), max: Num::Min(&Num::Lit(2), &Num::ZoneSize(ZoneRef(Who::Me, Zone::Deck))) },
                into: 0,
                ..PickSpec::DEFAULT
            })),
                Step::after_damage(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), into: 1, ..LookSpec::DEFAULT })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(1)), cards: CardSel::All, position: DeckPosition::Top, order: DeckOrder::ChosenBy(Who::Me), ..PutIntoDeckSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
