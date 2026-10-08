//! Flutter Mane (TEF): Midnight Fluttering — as long as this Pokémon is in the
//! Active Spot, your opponent's Active Pokémon has no Abilities, except for
//! Midnight Fluttering. Hex Hurl — 90; put 2 damage counters on your
//! opponent's Benched Pokémon in any way you like.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, exemptPowerNames
//! ['Midnight Fluttering'], hand/discard powers locked too, error
//! BLOCKED_BY_ABILITY): strips Abilities from CheckPokemonPowersEffect and
//! throws on PowerEffect when the callback says so: this card's list (findCardList
//! throws INVALID_GAME_STATE when it is in none) must have this card as its
//! owner's Active, the checked card's list must be the opponent's Active, and
//! LOCKER_ABILITY_APPLIES (activation-order check against other ability
//! lockers, then a real PowerEffect for Midnight Fluttering by the owner that
//! must not throw). The name exemption doesn't apply to a lock probe (its
//! power is named 'test'). Fixed (R1-14, ruling 1877): the callback returns
//! `false` for a checked card that has an Ability named "Hide 'n' Sneak" (it
//! takes precedence over Midnight Fluttering, whichever came into play
//! first; same fix in R2 and R4). Hex Hurl is PUT_X_DAMAGE_COUNTERS_IN_ANY_WAY_YOU_LIKE (2, Bench).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "FlutterMane",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::PlaceCountersAnyWay(PlaceCountersAnyWaySpec { among: SlotSel::Bench(Who::Opp), counters: 2, msg: "CHOOSE_POKEMON_TO_DAMAGE" })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::ActiveLock(ActiveLock::MidnightFluttering) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
