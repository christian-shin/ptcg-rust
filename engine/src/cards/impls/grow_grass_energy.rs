//! Growing [G] Energy ("Grow [G] Energy M3", POR): provides [G]. The [G]
//! Pokémon this card is attached to gets +20 HP.
//!
//! Twinleaf: the [G] entry is pushed unconditionally (no EnergyEffect probe).
//! On CheckHp, if the target holds this card and the special energy isn't
//! blocked, a CheckPokemonType on the target decides: [G] → hp += 20 (the
//! setter writes hpBonus only when a Pokémon was captured).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "GrowGrassEnergy",
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::ProvidesEnergy(ProvidesEnergySpec { entries: &[ProvidedEntry::always(&[ct::GRASS])], probe: false }) },
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::HpMod(HpModSpec { amount: 20, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::GRASS)]), guard: Cond::True }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
