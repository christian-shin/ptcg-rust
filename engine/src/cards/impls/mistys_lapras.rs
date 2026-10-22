//! Misty's Lapras (DRI 50): Swim Together - search your deck for up to 3 Misty's Pokémon, reveal them, and put them into your hand. Then, shuffle your deck. Surf - 60.
//!
//! Search of the deck for a kind of card (Pokémon with the Misty's tag), at most 3, revealed to the opponent, then one
//! shuffle (Cynthia's Gabite's Champion's Call).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MistysLapras",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Search(SearchSpec {
                    pick: PickSpec {
                        predicate: Pred::All(&[Pred::Pokemon, Pred::Tag(crate::types::tag::MISTYS)]),
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                        ..PickSpec::DEFAULT
                    },
                    destination: SearchDestination::Hand { reveal: true },
                    cancel: false,
                })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
