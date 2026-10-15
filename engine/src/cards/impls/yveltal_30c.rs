//! Yveltal (30C 100): Life-Locked — your opponent's Active Pokémon can't be healed. Dark Cutter — 90.
//!
//! A `Prevent` over the RemoveCounters (heal) events of the opponent's Active Pokémon, held while this Pokémon's Ability works, for
//! either player.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yveltal30C",
    // Your opponent's Active Pokémon can't be healed: a `Prevent` over RemoveCounters.
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::IsActive, SlotPred::Not(&SlotPred::OnMySide)]), EventPred::Kind(EventKind::RemoveCounters))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
