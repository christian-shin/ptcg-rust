//! Sacred Ash (DRI, as Sacred Ash FA POR): shuffle up to 5 Pokémon from your
//! discard pile into your deck.
//!
//! Twinleaf (destined-rivals file): `min 1, max min(5, Pokémon in discard)`, not cancellable since
//! phase 4b (it was: a cancel is choosing 0, rulings 1778/1853); the final
//! ShuffleDeckPrompt has no trailing wait. Phase 4b (R4, Meta-Rulings): the chosen
//! Pokémon are revealed to the opponent before they are moved (discard pile to
//! deck); up to 5 / min 1 is right (erratum, Rulings Compendium 1689).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SacredAsh@POR",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon), &Num::Lit(5)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Deck { reveal: true },
                msg: "",
                cancel: false,
            })),
            Step::new(Op::If(IfSpec { cond: Cond::Chosen(0), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
