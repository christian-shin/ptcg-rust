//! Volbeat (TWM 9): Quick Sign (usable on the first turn when going first) —
//! search your deck for up to 2 Basic Pokémon and put them onto your Bench,
//! then shuffle. Coordinated Strike — 20+; 60 more damage if Illumise is on
//! your Bench.
//!
//! Twinleaf: Quick Sign does nothing (no shuffle) with an empty deck or a full
//! Bench; else SEARCH_YOUR_DECK_FOR_POKEMON_AND_PUT_ONTO_BENCH({ stage: BASIC },
//! { min: 0, max: min(2, open slots) }) (cancellable).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VolbeatTWMPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Lit(2), &Num::OpenBench(Who::Me)) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Bench,
                        msg: "",
                        cancel: true,
                        shuffle_first: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
                ],
                no: &[],
            }))],
        },
        // Coordinated Strike: 60 more damage if Illumise is on your Bench.
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(60, Cond::InPlay(Who::Me, PlayScope::Bench, Pred::Name("Illumise"))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
