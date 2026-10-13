//! Team Rocket's Energy (DRI, Special): can only be attached to a Team
//! Rocket's Pokémon (discarded from anything else); provides 2 in any
//! combination of [P] and [D].
//!
//! Attaching it to a Pokémon that isn't a Team Rocket's Pokémon is refused (its `AttachGuard`); the state check's
//! table-state step discards a copy that is off one (unless its text is off, `derived::special_energy_off`). The two
//! [P]/[D] entries are added while its text is on (`ProvidesEnergySpec::probe`, read through
//! `derived::special_energy_off`); otherwise the core adds the printed [C].
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
