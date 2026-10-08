//! Gastrodon (SSP): Sticky Bind — as long as this Pokémon is on your Bench,
//! Benched Stage 2 Pokémon (both yours and your opponent's) have no
//! Abilities. Mud Shot — 80.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, allowUseFromHand,
//! allowUseFromDiscard, error BLOCKED_BY_ABILITY): strips Abilities from
//! CheckPokemonPowersEffect and throws on PowerEffect when this card is the
//! top card of a Bench slot of either player, the checked card is Stage 2,
//! its list (findCardList, which throws INVALID_GAME_STATE when the card is
//! in no list) is a Pokémon slot other than either Active, and this card's
//! ability isn't blocked for the player whose Bench holds it.
use crate::spec::prelude::*;
use crate::types::Stage;

pub static SPEC: CardSpec = CardSpec {
    class: "Gastrodon@SSP",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // While on your Bench: Benched Stage 2 Pokémon (both players') have no Abilities.
        modifier: Modifier::AbilityLock(AbilityLockSpec {
            error: "BLOCKED_BY_ABILITY",
            locker: Locker::BenchOfEitherSide,
            card: Pred::StageIs(Stage::Stage2),
            slot: SlotPred::IsBench,
            missing: Pred::False,
            powers: LockedPowers { generic_probe: true, only_knocks_out_self: false, exempt_name: None },
            probe: LockerProbe::Generic,
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
