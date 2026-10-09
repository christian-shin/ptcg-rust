//! Rock Fighting Energy (POR, "Rocky Fighting Energy"): provides [F]. Prevent
//! all effects of attacks used by your opponent's Pokémon done to the [F]
//! Pokémon this card is attached to (damage is not an effect).
//!
//! Twinleaf: the [F] entry is pushed on every CheckProvidedEnergyEffect of a
//! slot holding this card. Like Mist Energy, every AbstractAttackEffect on a
//! slot holding this card whose top Pokémon is printed [F] is prevented
//! unless it is ApplyWeakness / PutDamage / DealDamage, when its source slot
//! belongs to the target owner's opponent and holds a Pokémon; the
//! special-energy block probe runs first, for the target owner's opponent.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "RockFightingEnergy",
    passives: &[
        // An opponent's attack switching the [F] Pokémon this card is attached to in or out (ChangeActive: APR C-04 / C-05, id2025, id2155).
        Passive { origin: RuleSource::Energy, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::PrintedTypeIs(ct::FIGHTING)]), CHANGE_ACTIVE_BY_OPP_ATTACK)) },
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::FIGHTING])], probe: false }) },
        // Prevent all effects of attacks used by your opponent's Pokémon done to the [F] Pokémon this is attached to.
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::PreventAttackEffects(PreventAttackEffectsSpec {
                subject: SlotPred::All(&[SlotPred::Holder, SlotPred::PrintedTypeIs(ct::FIGHTING)]),
                probe_for_attacker: true,
                ..PreventAttackEffectsSpec::DEFAULT
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
