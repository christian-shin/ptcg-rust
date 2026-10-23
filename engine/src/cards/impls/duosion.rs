//! Duosion (BLK 38): Cellular Evolution - search your deck for a card that evolves from 1 of your Pokémon and put it onto that
//! Pokémon to evolve it. Then, shuffle your deck. Spray Fluid - 30.
//!
//! With a card in the deck, one Pokémon card that evolves from one of your Pokémon in play is chosen (the choice can be declined: a
//! search for a kind, APR E-19) and put onto that Pokémon by the attack: an Evolve event from the deck on the effect path, which no
//! first-turn or came-into-play-this-turn limit stops (APR C-12; Salvatore TEF, Team Rocket's Pupitar DRI 95); the shuffle follows.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Duosion",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFromOwnInPlay]),
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    cancel: true,
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::FromRegister { chooser: Who::Me, card: 0 } })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
