//! Salvatore (TEF, Supporter): search your deck for a card that has no Abilities and evolves from 1 of your Pokémon, and put
//! it onto that Pokémon to evolve it. Then, shuffle your deck. You can use this card on a Pokémon you put down when you were
//! setting up to play or on a Pokémon that was put into play this turn.
//!
//! The deck Pokémon must evolve from one of your Pokémon in play and print no Ability (the choice can be declined: a search
//! for a kind). The evolving is an Evolve event from the deck caused by the Supporter, with no restriction on a Pokémon put
//! into play this turn or at setup; Eevee ex's Rainbow DNA doesn't apply (not played from the hand, docs/rulings/RULES.md).
//! The shuffle follows (APR E-19). It needs a card in the deck and a Pokémon in play it could evolve (id925, id840).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Salvatore",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::HasEvolutionInPool(Who::Me)],
        steps: &[
            // A Pokémon without an Ability that evolves from 1 of your Pokémon in play (the choice can be cancelled).
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFromOwnInPlay, Pred::Not(&Pred::PrintsAbility)]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                cancel: true,
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::FromRegister { chooser: Who::Me, card: 0 } })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
