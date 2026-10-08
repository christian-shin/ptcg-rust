//! Energy Switch (SVI): move a Basic Energy from 1 of your Pokémon to
//! another of your Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "EnergySwitch",
    // Move a Basic Energy from 1 of your Pokémon to another of your Pokémon.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::MoveEnergyOwn(MoveEnergyOwnSpec { to: None, energy: Pred::BasicEnergy, cancel: false, used_always: false }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
