//! Meganium (M1S / MEG 10): Wild Growth - each Basic [G] Energy attached to your Pokémon provides [G][G] Energy (only 1 Wild Growth
//! at a time). Solar Beam - 140.
//!
//! `ProvidesEnergyBoost` over the Basic Energy cards that provide [G] (a Special Energy that provides [G], like Growing Grass
//! Energy, isn't boosted), on your Pokémon while the Ability works; it doesn't stack (APR: effects of the same Ability, JP FAQ
//! Meganium) and the doubled Energy counts in units for every "Energy attached" read.
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
