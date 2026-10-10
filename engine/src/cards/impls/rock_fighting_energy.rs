//! Rocky Fighting Energy (POR 87): provides [F]. Prevent all effects of attacks
//! used by your opponent's Pokémon done to the [F] Pokémon this card is attached
//! to. (Existing effects are not removed. Damage is not an effect.)
//!
//! Rule: a `Prevent(EFFECTS_OF_OPP_ATTACKS)` on the holder while it is a [F] Pokémon (its current type, APR D-07: a
//! Pokémon's type can change, e.g. Scovillain ex's Double Type; the same read as Shadowy Darkness Energy and Punk Helmet):
//! one declaration that ranges over every event with an effect (Special Conditions,
//! counters placed or moved onto it, discards, switches, devolving, a lasting effect
//! put on it, a Knock Out by an effect), never Damage (APR C-17, C-04, C-05; id2025,
//! id2155, id2341).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "RockFightingEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::FIGHTING])], probe: false }) },
        // Prevent all effects of attacks used by your opponent's Pokémon done to the [F] Pokémon this is attached to (every
        // event they cause, the switches included: APR C-04 / C-05, id2025, id2155; damage is not an effect).
        Passive { origin: RuleSource::Energy, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::FIGHTING)]), EFFECTS_OF_OPP_ATTACKS)) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
