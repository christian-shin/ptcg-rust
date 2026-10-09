//! Grand Tree ("Great Tree SCR", ACE SPEC stadium): once during each
//! player's turn, that player may search their deck for a Stage 1 Pokémon
//! that evolves from 1 of their Basic Pokémon and put it onto that Pokémon to
//! evolve it; if they did, they may search for a Stage 2 that evolves from
//! it and evolve again. Then, that player shuffles their deck. (Players can't
//! evolve a Basic Pokémon during their first turn or a Basic Pokémon that was
//! put into play this turn.)
//!
//! The "can evolve" test uses every non-Basic card the game knows
//! (`gen::evolutions::ALL_EVOLUTIONS`, id2030) whose evolvesFrom names an
//! in-play Basic. The deck prompts (cancellable) filter on Stage 1 / Stage 2
//! and the card evolving from the Pokémon (`enter::evolves_into`). The deck is
//! shuffled at every exit after the Pokémon prompt.
//!
//! Events batch 2: each evolving is the Evolve event's effect path from the
//! deck (not played from the hand: id1133, id2037). The printed parenthetical
//! is reminder text for the rule's limits (id2327), declared as the card's
//! `Limits` on its first evolving (a Basic Pokémon): the effect path is subject
//! to them as an evolution from the hand is, and a permission lifts them
//! (Eevee's Boosted Evolution while it is the Active Pokémon). Rare Candy's
//! printed limit is a `Restrict` instead, which nothing lifts (id1144, id1815).
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "GreatTree",
    use_stadium: Some(PlaySpec {
        kind: PlayKind::Stadium,
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::CanEvolveBasic(Who::Me)],
        steps: &[Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::FromDeck { chooser: Who::Me, stage: Stage::Stage1, then_stage: Some(Stage::Stage2) } })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))],
    }),
    // Players can't evolve a Basic Pokémon during their first turn or a Basic Pokémon that was put into play
    // this turn.
    limits: &[Limits {
        on: EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::CauseCard), EventPred::Base(Pred::Basic)]),
        limits: &[Limit::FirstTurn, Limit::BaseEnteredThisTurn],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
