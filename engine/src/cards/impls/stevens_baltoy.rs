//! Steven's Baltoy (DRI): Summoning Sign — search your deck for up to 2 Basic
//! Steven's Pokémon and put them onto your Bench. Then, shuffle your deck.
//! Psychic Sphere — 20.
//!
//! Twinleaf: no-op without a free Bench slot; otherwise a ChooseCardsPrompt on
//! the deck (Basic Pokémon, 0..min(slots, 2), no cancel) with every non-Steven's
//! card blocked by deck index; the callback plays each card into the slot it
//! saw at prompt time, then SHUFFLE_DECK (even when nothing was chosen).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StevensBaltoy",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::BenchSpace(Who::Me),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            chooser: Who::Me,
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::Basic, Pred::Tag(crate::types::tag::STEVENS)]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                        },
                        destination: SearchDestination::Bench,
                        msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH",
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
                ],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
