//! Bubbly Water Energy ("Bubble Water Energy M4", CRI): provides [W]. The
//! [W] Pokémon this card is attached to recovers from all Special Conditions
//! and can't be affected by any Special Conditions.
//!
//! Twinleaf: on its AttachEnergyEffect (before the card is attached) a [W]
//! target loses all five conditions unless the special energy is blocked.
//! PREVENT_AND_CLEAR_SPECIAL_CONDITIONS: attack and power AddSpecialConditions
//! effects on a [W] slot holding this card are prevented, and every
//! CheckTableState clears the conditions of each [W] slot holding it (both
//! unless blocked for the slot's owner). Events batch 4: a `Prevent` over
//! GainCondition (any cause) and `Recover` at the table check.
//!
//! Fixed (phase 4b, W4): there was no [W] type check anywhere (the
//! CheckPokemonType on attach was computed and ignored), so any Pokémon
//! holding the card was immune.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BubbleWaterEnergy",
    // The [W] Pokémon this card is attached to recovers from all Special Conditions and can't be
    // affected by any Special Conditions.
    passives: &[
        Passive { origin: RuleSource::Energy, modifier: Modifier::Recover(RecoverSpec { conds: &[], subject: SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::WATER)]) }) },
        Passive {
            origin: RuleSource::Energy,
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::TypeIs(ct::WATER)]), EventPred::Kind(EventKind::GainCondition))),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
