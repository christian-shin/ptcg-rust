//! N's Purrloin (JTG): Thieving Swipe — 30; your opponent reveals their hand, and you choose a card you find there and put it
//! on the bottom of their deck (a PutIntoDeck of the attack's effect, from the opponent's hand to the opponent's deck).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "NsPurrloin",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: false })),
            Step::after_damage(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
