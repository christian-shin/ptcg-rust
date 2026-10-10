//! Metang (TEF): Metal Maker — once during your turn, look at the top 4 cards
//! of your deck and attach any number of [M] Energy you find there to your
//! Pokémon; put the rest on the bottom of your deck.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Metang",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("METAL_MAKER_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(4)), into: 0, ..LookSpec::DEFAULT })),
            Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Scratch(0)),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Lit(4), &Num::CardCount(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::BasicEnergy)) },
                valid_types: &[crate::types::ct::METAL],
                route: AttachRoute::Effect,
                ..AttachSpec::DEFAULT
            })),
            // The rest goes to the bottom of the deck, shuffled first.
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
