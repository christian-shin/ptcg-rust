//! Gothorita (WHT 42): Fortunate Eye - look at the top 5 cards of your opponent's deck and put them back in any order. Psyshot - 40.
//!
//! The top 5 cards of the opponent's deck (all of it when fewer) are set aside for the player to look at, then put back on top of
//! that deck in the order the player chooses (`DeckOrder::ChosenBy`; Deduction Kit SSP 171's put-back). An empty deck: nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Gothorita",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Look(LookSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(5)), viewer: Who::Me, into: 0 })),
            Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Top, order: DeckOrder::ChosenBy(Who::Me), ..PutIntoDeckSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
