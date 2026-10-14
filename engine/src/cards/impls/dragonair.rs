//! Dragonair (ASC 151): Evolutionary Guidance: once during your turn, if this Pokémon has any Energy attached, you may
//! search your deck for an Evolution Pokémon, reveal it, and put it into your hand. Then, shuffle your deck. Tail Snap: 60.
//!
//! An Ability (once per turn): needs this Pokémon to have Energy attached; the chosen Evolution Pokémon (any non-Basic Pokémon)
//! is revealed and goes into the hand (a PutIntoHand event, this Ability its cause), then the deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dragonair",
    // Evolutionary Guidance: once during your turn, if this Pokémon has any Energy attached, search
    // your deck for an Evolution Pokémon, reveal it, put it into your hand and shuffle.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::Slot(SlotExpr::This, SlotPred::HasEnergy)],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                cancel: true,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
