//! Meganium (M1S): Wild Growth — each Basic [G] Energy attached to your
//! Pokémon provides [G][G] Energy (only 1 Wild Growth at a time). Solar Beam — 140.
//!
//! Twinleaf: on every CheckProvidedEnergyEffect whose player has this card
//! as the top card of an in-play Pokémon (and the ability isn't blocked),
//! every Basic Energy card in the source's `cards` providing [G] not yet in
//! the map gets a [G][G] entry. Extra copies find their entries already
//! mapped.
//!
//! Fixed (phase 4b, W4): Twinleaf also doubled Special Energy providing [G]
//! (e.g. Growing Grass Energy); the card says Basic [G] Energy only.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "Meganium",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Each Basic [G] Energy attached to your Pokémon provides [G][G].
        modifier: Modifier::ProvidesEnergyBoost(ProvidesEnergyBoostSpec {
            energy: Pred::All(&[Pred::BasicEnergy, Pred::ProvidesType(ct::GRASS)]),
            provides: &[ct::GRASS, ct::GRASS],
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
