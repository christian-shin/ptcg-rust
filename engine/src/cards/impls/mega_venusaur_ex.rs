//! Mega Venusaur ex (M1L 3): Solar Trans — as many times as you like during
//! your turn, move a Basic [G] Energy from one of your Pokémon to another.
//! Jungle Dump — 240; heal 30 damage from this Pokémon.
//!
//! Solar Trans has no limit (`Once::No`): each use moves one Basic [G] Energy from one of your Pokémon to another (a MoveEnergy
//! event, cause: this Ability); a Special Energy can't be moved (JP FAQ Mega Venusaur ex).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaVenusaurEx",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(30))] }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::MoveEnergyOwn(MoveEnergyOwnSpec { to: None, energy: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]), cancel: false }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
