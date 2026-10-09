//! Team Rocket's Watchtower (DRI, stadium): [C] Pokémon in play (both
//! yours and your opponent's) have no Abilities.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, allowUseFromHand and
//! allowUseFromDiscard, error BLOCKED_BY_EFFECT): strips Abilities from
//! CheckPokemonPowersEffect and throws on PowerEffect. The lock applies to a
//! card on a Pokémon slot whose CheckPokemonTypeEffect includes [C] (unless
//! stadium effects on that slot are blocked); a card not found in any list
//! falls back to its printed types.
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
                powers: LockedPowers { generic_probe: true, only_knocks_out_self: false, exempt_name: None },
                probe: LockerProbe::StadiumOnSlot,
            }),
        },
        Passive { origin: RuleSource::Stadium, modifier: Modifier::BlockUse(BlockUseSpec::USE_STADIUM) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
