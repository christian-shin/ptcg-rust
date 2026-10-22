//! Waitress (ASC 215, Supporter): look at the top 6 cards of your deck and attach a Basic Energy card you find there to 1 of your
//! Pokémon. Shuffle the other cards back into your deck.
//!
//! The 6 cards are set aside (a Look); a Basic Energy card among them is attached to the Pokémon of your choice (an Attach event
//! caused by the Supporter; the attachment is not the turn's Energy attachment), and the rest go back to the deck, which is then
//! shuffled.
use crate::spec::prelude::*;

const LOOKED: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(0));

pub static SPEC: CardSpec = CardSpec {
    class: "Waitress",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(6)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Attach(AttachSpec {
                from: LOOKED,
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::ActiveBench,
                bounds: Bounds { min: Num::Min(&Num::Lit(1), &Num::CardCount(LOOKED, Pred::BasicEnergy)), max: Num::Lit(1) },
                ..AttachSpec::DEFAULT
            })),
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: LOOKED, cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
