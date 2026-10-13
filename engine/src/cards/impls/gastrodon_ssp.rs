//! Gastrodon (SSP 107): Sticky Bind — as long as this Pokémon is on your Bench, Benched Stage 2 Pokémon (both yours and your
//! opponent's) have no Abilities. Mud Shot — 80.
//!
//! `Modifier::AbilityLock` (locker: the Bench of either side): every Stage 2 Pokémon on a Bench has no Abilities while this
//! Ability works. It is an Ability, so another lock on Gastrodon turns it off; the Active Pokémon is never covered.
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
