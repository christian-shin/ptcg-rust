//! Patrat (M4): Watchful Eye — damage counters on each Pokémon can't be
//! moved to other Pokémon. Bite — 10.
//!
//! Twinleaf: any MoveDamageCountersEffect or MoveCountersAttackEffect (Cofagrigus
//! WHT's Extended Damagriiigus) is prevented while any Patrat is in play on
//! either side (checked from any zone).
//!
//! Fixed (phase 4b, R2): a Patrat whose Ability is blocked (Team Rocket's
//! Watchtower hits [C] Pokémon) still prevented the move; each Patrat now
//! counts only when `IS_ABILITY_BLOCKED` is false for it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Patrat",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec { what: PreventWhat::CounterMoves }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
