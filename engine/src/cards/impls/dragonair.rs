//! Dragonair (M2a / ASC 151): Evolutionary Guidance — once during your turn,
//! if this Pokémon has any Energy attached, search your deck for an Evolution
//! Pokémon (non-Basic Pokémon; every other card is blocked by deck index),
//! reveal it, put it into your hand and shuffle. Tail Snap — 60.
//!
//! Twinleaf order: ability-blocked probe, in-play lookup, energy check,
//! marker check; then SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_INTO_HAND (throws
//! on an empty deck before the marker is set), marker, board effect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dragonair",
    // Evolutionary Guidance: once during your turn, if this Pokémon has any Energy attached, search
    // your deck for an Evolution Pokémon, reveal it, put it into your hand and shuffle.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("EVOLUTION_GUIDANCE_MARKER"),
        needs: &[Cond::Slot(SlotExpr::This, SlotPred::HasEnergy)],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: true,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
