//! Hoothoot (PRE 77): Insomnia - this Pokémon can't be Asleep. Tackle - 20.
//!
//! Insomnia is a `Prevent` over the GainCondition event naming Asleep, on this Pokémon only (`Holder` and `IsThisPokemon`),
//! whatever causes the condition (an attack, a Trainer, an Ability, a rule). While its Ability is locked it prevents nothing, and
//! a Hoothoot already Asleep when it is locked stays Asleep (scenarios hoothoot-pre-77-insomnia-*). This port is the PRE
//! card: the other Hoothoot printings (SCR, TEF) have no Ability.
use crate::spec::prelude::*;
use crate::types::SpecialCondition;

pub static SPEC: CardSpec = CardSpec {
    class: "HoothootPRE",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // This Pokémon can't be Asleep (any cause: events batch 4).
        modifier: Modifier::Prevent(PreventSpec::on(
            SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            EventPred::All(&[EventPred::Kind(EventKind::GainCondition), EventPred::Condition(SpecialCondition::Asleep)]),
        )),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
