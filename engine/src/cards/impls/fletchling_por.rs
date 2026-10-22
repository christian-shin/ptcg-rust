//! Fletchling (POR 66): Chirp - search your deck for up to 2 Pokémon with [F] Resistance, reveal them, and put them into
//! your hand. Then, shuffle your deck. Peck - 20.
//!
//! With a card in the deck, one search of 0 to 2 Pokémon whose printed Resistance includes [F] (`Pred::HasResistance`),
//! revealed and put into the hand, then one shuffle (APR E-19; Celebi MEG 12).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FletchlingPOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                    yes: &[
                        Step::new(Op::Search(SearchSpec {
                            pick: PickSpec {
                                predicate: Pred::All(&[Pred::Pokemon, Pred::HasResistance(ct::FIGHTING)]),
                                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                                ..PickSpec::DEFAULT
                            },
                            destination: SearchDestination::Hand { reveal: true },
                            cancel: false,
                        })),
                        Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                    ],
                    no: &[],
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
