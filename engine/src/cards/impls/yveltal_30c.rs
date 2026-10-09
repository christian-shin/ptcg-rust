//! Yveltal (30C 100): Life-Locked — your opponent's Active Pokémon can't be
//! healed. Dark Cutter — 90.
//!
//! Fixed in phase 4b (R4): the HealEffect handler loops over `state.players`
//! and skips a player who does not have this card in play (it used to return
//! at the first such player, so the Ability only worked when this card
//! belonged to the first player, index 0). A blocked Ability skips that
//! player's turn of the loop too.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Yveltal@30C",
    // Your opponent's Active Pokémon can't be healed: a `Prevent` over RemoveCounters (events batch 4).
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::IsActive, SlotPred::Not(&SlotPred::OnMySide)]), EventPred::Kind(EventKind::RemoveCounters))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
