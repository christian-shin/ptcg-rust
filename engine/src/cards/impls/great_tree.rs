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
//! is the card's own restriction on its first evolving (a Basic Pokémon),
//! declared as a `Restrict` as Rare Candy's is: no permission lifts it, on
//! either clause (Eevee's Boosted Evolution, Forest of Vitality; official JP
//! Q&A 2026-10-09, docs/rulings/forum-answers.md; id1144, id1815). id2327
//! (a permission overrides the reminder text) is Strange Timepiece's only.
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
    restricts: &[Restrict {
        on: EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::CauseCard), EventPred::Base(Pred::Basic)]),
        limits: &[Limit::FirstTurn, Limit::BaseEnteredThisTurn],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
