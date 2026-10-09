//! Rare Candy (SVI): choose 1 of your Basic Pokémon in play; if you have a
//! Stage 2 card in your hand that evolves from it, put it onto that Pokémon.
//! (This counts as evolving that Pokémon.) You can't use this card during your
//! first turn or on a Basic Pokémon that was put into play this turn.
//!
//! Events batch 2: the evolving is the Evolve event's rule path from the hand
//! (it counts as playing the card from the hand: id285, id1998; locks on
//! evolving from the hand stop it: id1133). Its printed limit is its own
//! `Restrict`, which no permission lifts (Forest of Vitality, Boosted
//! Evolution: id1144, id1815; docs/rulings/RULES.md).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RareCandy",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::RareCandyUsable],
        steps: &[Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::RareCandy }))],
    }),
    // You can't use this card during your first turn or on a Basic Pokémon that was put into play this turn.
    restricts: &[Restrict { on: EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::CauseCard)]), limits: &[Limit::FirstTurn, Limit::BaseEnteredThisTurn] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
