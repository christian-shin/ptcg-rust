//! Voltaic [L] Energy ("Bolty [L] Energy M5", PBL): provides [L]. Attacks used
//! by the [L] Pokémon this card is attached to do 20 more damage to your
//! opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! Twinleaf: the [L] entry is pushed unless an EnergyEffect probe throws. On a
//! DealDamageEffect whose source slot holds this card (unless the special
//! energy is blocked): a CheckPokemonType on the source must contain [L];
//! then `damage > 0 && target === opponent.active` → damage += 20.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "VoltaicLightningEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::LIGHTNING])], probe: true }) },
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
