//! Sacred Ash (POR 115, as in DRI): Item. Shuffle up to 5 Pokémon from your discard pile into your deck.
//!
//! The erratum is "up to 5" (id2176): an Item's "up to" is 1 to 5 (id2301). The chosen Pokémon (revealed by the move, id787)
//! are put into the deck by one PutIntoDeck event from the discard pile (cause: this Item), then the deck is shuffled.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SacredAsh@POR",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon), &Num::Lit(5)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Deck { reveal: false },
                cancel: false,
            })),
            Step::new(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
