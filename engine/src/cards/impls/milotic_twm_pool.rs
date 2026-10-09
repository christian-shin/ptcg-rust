//! Milotic (TWM 50): Mentally Calm — your opponent's Pokémon in play and all
//! attached cards can't be put into your opponent's hand. Hydro Splash — 100.
//!
//! Twinleaf: reacts to every MoveCardsEffect whose source is a Pokémon slot
//! (any card of any copy): this card must be the top Pokémon of a slot (found
//! with findCardList; any failure returns), the destination must be the
//! hand of its owner's opponent, the source slot must be one of that
//! opponent's occupied Pokémon slots, and the owner's generic Ability probe
//! must pass; then preventDefault.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MiloticTWMPool",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::MoveToHandFromOppPlay, ..PreventSpec::NONE }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
