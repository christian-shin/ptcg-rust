//! Team Rocket's Watchtower (ASC 210, Stadium): [C] Pokémon in play (both yours and your opponent's) have no Abilities.
//!
//! An Ability lock (`Modifier::AbilityLock`, the Stadium in play as the locker): it strips and blocks the Abilities of every Pokémon in
//! play whose type, read after effects, is [C]; the Stadium's own effect must not be blocked on that Pokémon's spot
//! (`LockerProbe::StadiumOnSlot`).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsWatchtower",
    passives: &[
        // [C] Pokémon in play (both players') have no Abilities.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::AbilityLock(AbilityLockSpec {
                error: "BLOCKED_BY_EFFECT",
                locker: Locker::StadiumInPlay,
                card: Pred::Any,
                slot: SlotPred::TypeIs(ct::COLORLESS),
                missing: Pred::PrintedType(ct::COLORLESS),
                powers: LockedPowers { generic_probe: true, only_knocks_out_self: false },
                probe: LockerProbe::StadiumOnSlot,
            }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
