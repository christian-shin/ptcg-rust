//! Energy Swatter (POR): your opponent reveals their hand, and you choose an
//! Energy card you find there and put it on the bottom of their deck.
//!
//! Twinleaf: a no-op hand→discard MOVE_CARDS of the card (it already sits in
//! the supporter pile), the reveal to the player is queued without waiting,
//! and with no Energy in the opponent's hand nothing else happens.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EnergySwatter",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any)],
        steps: &[
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(ZoneRef(Who::Opp, Zone::Hand)), by: Who::Opp, to: Who::Me, when_empty: false })),
            Step::new(Op::Pick(PickSpec { chooser: Who::Me, from: ZoneRef(Who::Opp, Zone::Hand), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, soft: true, ..PickSpec::DEFAULT })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Chosen(0), position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
