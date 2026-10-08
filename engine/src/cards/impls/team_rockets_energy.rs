//! Team Rocket's Energy (DRI, Special): can only be attached to a Team
//! Rocket's Pokémon (discarded from anything else); provides 2 in any
//! combination of [P] and [D].
//!
//! Twinleaf: attaching to a slot whose top card is not a Team Rocket's
//! Pokémon throws. The table-state discard skips copies whose
//! SpecialEnergyEffect probe (for the slot's owner) is blocked. The two
//! [P]/[D] entries are only added if the EnergyEffect probe (for the checked
//! player) passes; otherwise the core adds the printed [C].
use crate::spec::prelude::*;
use crate::types::{ct, tag};

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsEnergy",
    passives: &[
        // 2 in any combination of [P] and [D].
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::PSYCHIC, ct::DARK]), ProvidedEntry::always(&[ct::PSYCHIC, ct::DARK])], probe: true }),
        },
        // Can only be attached to a Team Rocket's Pokémon.
        Passive { origin: RuleSource::Energy, modifier: Modifier::AttachGuard(AttachGuardSpec { allow: SlotPred::Tag(tag::TEAM_ROCKET) }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
