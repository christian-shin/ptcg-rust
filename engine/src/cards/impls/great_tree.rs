//! Grand Tree ("Great Tree SCR", ACE SPEC stadium): once during each
//! player's turn, that player may search their deck for a Stage 1 Pokémon
//! that evolves from 1 of their Basic Pokémon and put it onto that Pokémon to
//! evolve it; if they did, they may search for a Stage 2 that evolves from
//! it and evolve again. Then, that player shuffles their deck.
//!
//! Twinleaf quirks kept: the "can evolve" test uses every non-Basic card the
//! CardManager knows (`gen::evolutions::ALL_EVOLUTIONS`) whose evolvesFrom
//! names an in-play Basic not put into play this turn (CheckPokemonPlayedTurn).
//! Fixed in phase 4b (R4): a Pokémon can't be evolved during its owner's
//! first turn (turn <= 2 without `canEvolve`, unless the CheckPokemonPlayedTurn
//! effect grants `canEvolveOnFirstTurn`, as Eevee's Boosted Evolution does),
//! the same test as PlayPokemonEffect; it used to be missing. Only non-Basics,
//! Basics played this turn and first-turn Basics are blocked in the Pokémon
//! prompt. The deck prompts (cancellable, deck
//! Pokémon with another evolvesFrom blocked) filter on Stage 1 / Stage 2 and
//! evolvesFrom. The second prompt appears whenever any known card evolves
//! from the chosen Stage 1. Evolving is MOVE_CARDS deck→slot + clearEffects
//! + pokemonPlayedTurn = turn (no EvolveEffect). The deck is shuffled at
//! every exit after the Pokémon prompt.
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "GreatTree",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::CanEvolveBasic(Who::Me)],
        steps: &[Step::new(Op::Evolve(EvolveSpec { chooser: Who::Me, stage: Stage::Stage1, then_stage: Some(Stage::Stage2) })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
