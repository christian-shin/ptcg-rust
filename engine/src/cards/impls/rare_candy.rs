//! Rare Candy (SVI): choose 1 of your Basic Pokémon in play; if you have a
//! Stage 2 card in your hand that evolves from it, put it onto that Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RareCandy",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::RareCandyUsable],
        steps: &[Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::RareCandy }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
