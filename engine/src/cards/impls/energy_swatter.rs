//! Energy Swatter (POR 73): your opponent reveals their hand, and you choose an Energy card you find there and put it on the
//! bottom of their deck.
//!
//! The reveal is a routine (no card reacts to it); the card you choose goes from their hand to the bottom of their deck: one
//! PutIntoDeck event (position Bottom) on the opponent's side. With no Energy there nothing is chosen.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EnergySwatter",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: false })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, soft: true, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
