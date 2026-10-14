//! Psyduck (ASC 39 / MEP 7): Damp - Pokémon in play (both yours and your opponent's) lose any Ability that requires the Pokémon
//! using it to Knock Out itself. Ram - 20.
//!
//! Damp is an `AbilityLock` (`DAMP`) over the Pokémon of both players: the Abilities that need their Pokémon to Knock itself
//! Out are off while Damp works, Damp itself excepted (JP FAQ Dusknoir / Dusclops, id2043).
use crate::spec::prelude::*;

/// Damp (Psyduck, Golduck): Pokémon in play lose Abilities that require them to Knock Out themselves.
pub const DAMP: AbilityLockSpec = AbilityLockSpec {
    error: "BLOCKED_BY_ABILITY",
    locker: Locker::InPlayEitherSide,
    card: Pred::Any,
    slot: SlotPred::Any,
    missing: Pred::False,
    powers: LockedPowers { generic_probe: false, only_knocks_out_self: true },
    probe: LockerProbe::OwnPower(0),
};

pub static SPEC: CardSpec = CardSpec {
    class: "Psyduck@ASC|MEP",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AbilityLock(DAMP) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
