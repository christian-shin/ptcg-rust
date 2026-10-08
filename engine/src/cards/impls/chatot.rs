//! Chatot (TEF): A Capella — search your deck for up to 3 Basic Pokémon and
//! put them onto your Bench, then shuffle. Gust — 20.
//!
//! Twinleaf: with no empty Bench slot the attack does nothing, with no search
//! and no shuffle (phase 4b R7E, ruling 337; it used to open a prompt with max
//! 0 and shuffle); no empty-deck check; the ShuffleDeckPrompt has no trailing
//! wait.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Chatot",
    // A Capella: search your deck for up to 3 Basic Pokémon and put them onto your Bench, then
    // shuffle (nothing happens with no empty Bench space).
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::BenchSpace(Who::Me),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Bench,
                    msg: "",
                    cancel: false,
                    shuffle_first: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
