//! Patrat (M4): Watchful Eye — damage counters on each Pokémon can't be
//! moved to other Pokémon. Bite — 10.
//!
//! A lock over the MoveCounters event, for both players, while the Ability works: a move of counters doesn't happen and
//! they stay where they are (id2350; official JP FAQ, Patrat x2). Every way counters move is one MoveCounters action
//! (an attack's or an Ability's: Alakazam TWM, Cofagrigus, Team Rocket's Wobbuffet, Munkidori's Adrena-Brain).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Patrat",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::BlockUse(BlockUseSpec { binds: Binds::Both, lock: LockDecl::on(EventPred::Kind(EventKind::MoveCounters), "BLOCKED_BY_ABILITY"), while_: &[], ability: true }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
