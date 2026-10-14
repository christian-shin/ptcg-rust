//! Hilda (WHT 84): Supporter. Search your deck for an Evolution Pokémon and an Energy card, reveal them, and put them
//! into your hand. Then, shuffle your deck.
//!
//! At most one Evolution Pokémon and one Energy card (the caps); the cards are revealed and put into the hand (a
//! PutIntoHand event), then the deck is shuffled once (APR E-19). It can't be played with an empty deck: you can't search
//! cards that aren't there (id840, id925).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hilda",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)]), Pred::Energy]), bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)])), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Energy), &Num::Lit(1))) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)])), &Num::Lit(1)) }, Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Energy), &Num::Lit(1)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
