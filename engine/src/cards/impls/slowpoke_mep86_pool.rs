//! Slowpoke (MEP 86): Dopey Face — this Pokémon can't be Confused. Super Psy Bolt — 50.
//!
//! A `Prevent` over the GainCondition event of Confused on this Pokémon, whatever causes it: Lisia's Appeal and Dangerous Laser are
//! stopped like an attack (no card is named). A Burned and Confused effect still Burns it.
use crate::spec::prelude::*;
use crate::types::SpecialCondition;

pub static SPEC: CardSpec = CardSpec {
    class: "SlowpokeMEP86",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // This Pokémon can't be Confused.
        modifier: Modifier::Prevent(PreventSpec::on(
            SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            EventPred::All(&[EventPred::Kind(EventKind::GainCondition), EventPred::Condition(SpecialCondition::Confused)]),
        )),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
