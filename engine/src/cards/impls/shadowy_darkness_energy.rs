//! Shadowy Darkness Energy ("Shadow Darkness Energy M5", PBL): provides [D].
//! Prevent all damage done by your opponent's attacks to the Benched [D]
//! Pokémon this card is attached to.
//!
//! Twinleaf: the [D] entry is pushed unless an EnergyEffect probe throws.
//! A DealDamage / PutDamage effect during the ATTACK phase on a benched slot
//! holding this card, from the slot owner's opponent, gets `damage = 0`
//! unless the special energy is blocked or a CheckPokemonTypeEffect on the
//! slot lacks [D].
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "ShadowyDarknessEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::DARK])], probe: true }) },
        // Prevent all damage done by your opponent's attacks to the Benched [D] Pokémon this is attached to.
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsBench, SlotPred::TypeIs(ct::DARK)]), DAMAGE_BY_OPP_ATTACKS)),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
