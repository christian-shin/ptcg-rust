//! Legacy Energy (TWM, ACE SPEC): provides every type of Energy but only 1
//! at a time. If the Pokémon this card is attached to is Knocked Out by
//! damage from an attack from your opponent's Pokémon, that player takes 1
//! fewer Prize card (once per game).
//!
//! Twinleaf: provides [ANY] on every CheckProvidedEnergyEffect of its slot.
//! On a KnockOutEffect of its slot during the ATTACK phase of the KO'd
//! Pokémon's opponent (any KO then, not only from damage), unless the
//! special energy is blocked, `prizeCount -= 1` once per game
//! (`player.legacyEnergyUsed`).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "LegacyEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::ANY])], probe: false }) },
        Passive { origin: RuleSource::Energy, modifier: Modifier::PrizeAdjustOnce(PrizeAdjustSpec { delta: -1, subject: SlotPred::Holder, by_attack_damage: true, by_own_attack: None, guard: Cond::True, ..PrizeAdjustSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
