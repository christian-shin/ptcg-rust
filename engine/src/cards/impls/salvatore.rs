//! Salvatore (TEF): search your deck for a Pokémon, except any Pokémon with
//! an Ability, that evolves from 1 of your Pokémon in play and put it on
//! that Pokémon to evolve it (also on the turn it was played), then shuffle.
//!
//! Twinleaf: evolution names come from every non-Basic card in the
//! CardManager whose `evolvesFrom` names one of your Pokémon in play (only
//! used for the "nothing can evolve" throw). Fixed (phase 4b): a deck Pokémon
//! is blocked when its `evolvesFrom` names no Pokémon in play, or when it has
//! an Ability (before, any Ability-less Pokémon was selectable and the second
//! prompt then had no valid target; Ability evolutions were selectable too).
//! The CheckPokemonPowers probe runs for every deck Pokémon. The card moves to the
//! supporter pile and the TrainerEffect is prevented before the deck check.
//! The evolution is a plain MOVE_CARDS deck→slot + clearEffects +
//! pokemonPlayedTurn = turn (no EvolveEffect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Salvatore",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::HasEvolutionInPool(Who::Me)],
        steps: &[
            // A Pokémon without an Ability that evolves from 1 of your Pokémon in play (the choice can be cancelled).
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFromOwnInPlay, Pred::Not(&Pred::PrintsAbility)]),
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                cancel: true,
                into: 0,
                msg: "CHOOSE_CARD_TO_EVOLVE",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::FromRegister { chooser: Who::Me, card: 0 } })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
