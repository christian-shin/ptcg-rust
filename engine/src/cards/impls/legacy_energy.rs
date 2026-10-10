//! Legacy Energy (TWM, ACE SPEC): provides every type of Energy but only 1
//! at a time. If the Pokémon this card is attached to is Knocked Out by
//! damage from an attack from your opponent's Pokémon, that player takes 1
//! fewer Prize card (once per game).
//!
//! A `PrizeAdjustOnce` (-1) over the KnockOut view: it applies to a KnockOut whose `ko_by` is AttackDamage (a Knock Out by
//! an effect or by Poison or Burn doesn't count, rule-E04) of the holder, once per game for the player the Energy
//! belongs to, while its Special Energy effect works.
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
