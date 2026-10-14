//! Voltaic Lightning Energy (PBL 84, Special Energy): provides [L]. Attacks used by the [L] Pokémon this card is attached to do 20 more
//! damage to your opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! A `ProvidesEnergy` of [L] and a `DamageDealt` modifier of 20 for the
//! attacks of the Holder while it is [L] (read after effects), only when the attack does damage.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "VoltaicLightningEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::LIGHTNING])] }) },
        // Attacks used by the [L] Pokémon this is attached to do 20 more damage to the opponent's Active.
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::DamageDealt(DamageDealtSpec {
                amount: 20,
                attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::LIGHTNING)]),
                side: Side::Any,
                needs_damage: true,
                ..DamageDealtSpec::DEFAULT
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
