//! Growing [G] Energy (POR 86, Special Energy): provides [G]. The [G] Pokémon this card is attached to gets +20 HP.
//!
//! `Modifier::ProvidesEnergy` ([G], always) and `Modifier::HpMod` (+20) over the Holder when it is [G] after effects
//! (`SlotPred::TypeIs`).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "GrowGrassEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::GRASS])] }) },
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::HpMod(HpModSpec { amount: 20, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::GRASS)]), guard: Cond::True }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
