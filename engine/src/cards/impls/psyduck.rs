//! Psyduck (ASC 39 / MEP 7): Damp — Pokémon in play (both yours and your
//! opponent's) lose any Ability that requires the Pokémon using it to Knock
//! Out itself. Ram — 20.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, onlyKnocksOutSelf,
//! exemptPowerNames ['Damp'], allowUseFromHand, allowUseFromDiscard, error
//! BLOCKED_BY_ABILITY): strips matching Abilities from
//! CheckPokemonPowersEffect and throws on PowerEffect when this card is in play
//! (any Pokémon card of either player, top card of a slot), the checked card is
//! in a Pokémon slot (findCardList failures count as not locked), and a real
//! PowerEffect for Damp by this card's owner doesn't throw. The generic
//! lock probe is never subject (it has no knocksOutSelf).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Psyduck@ASC|MEP",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AbilityLock(DAMP) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
