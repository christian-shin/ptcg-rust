//! Shadowy Darkness Energy ("Shadow Darkness Energy M5", PBL): provides [D]. Prevent all damage done by your opponent's
//! attacks to the Benched [D] Pokémon this card is attached to.
//!
//! The prevention is a `Prevent` over `Kind(Damage)` caused by the opponent's attacks (`DAMAGE_BY_OPP_ATTACKS`), read at
//! step 6 of the damage calculation (APR C-16, decision D7): a later "takes N more damage" can't undo it. It needs the
//! Special Energy to work and the Pokémon to be a [D] Pokémon on the Bench.
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
