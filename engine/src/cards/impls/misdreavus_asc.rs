//! Misdreavus (ASC 85): Ascension - search your deck for a card that evolves from this Pokémon and put it onto this Pokémon to evolve it. Then, shuffle your deck.
//!
//! An empty deck can't be searched (id840): nothing happens, no shuffle. The evolving is an Evolve event from the deck (not
//! played from the hand), caused by the attack; the choice can be declined (Dwebble DRI 11).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MisdreavusASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                    yes: &[
                        Step::new(Op::Pick(PickSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFromActive]),
                            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                            into: 0,
                            cancel: true,
                            ..PickSpec::DEFAULT
                        })),
                        Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot: MY_ACTIVE, card: 0 } })),
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
