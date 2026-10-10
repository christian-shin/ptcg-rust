//! Tatsugiri (TWM): Attract Customers - once during your turn, if this
//! Pokémon is in the Active Spot, look at the top 6 cards of your deck,
//! reveal a Supporter there and put it into your hand; shuffle the rest back.
//!
//! Twinleaf quirk kept: the Active check is `player.active.cards[0] === this`.
//! Fixed (phase 4b): the prompt cannot be cancelled (`min: 0` already allows
//! taking nothing; cancelling moved every looked-at card into the hand and
//! crashed), and the deck is shuffled after the reveal as well.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tatsugiri",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("CROWD_PULLER_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::IsActive(SlotExpr::This)],
        // Look at the top 6 cards of your deck, reveal a Supporter there and put it into your hand; shuffle the rest back.
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Scratch(0)),
                predicate: Pred::Supporter,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                into: 1,
                msg: "CHOOSE_CARD_TO_HAND",
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
