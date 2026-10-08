//! Relicanth (TEF): Memory Dive — each of your evolved Pokémon can use any
//! attack from its previous Evolutions. Razor Fin — 30.
//!
//! The Twinleaf class is `set-temporal-forces/relicanth.ts` (the one the TEF
//! set index registers; `data/pool.json` points at an unused TWM file).
//! * CheckTableStateEffect: `effect.player` is never set, so
//!   `owner !== player` always returns (after findCardList, which throws
//!   INVALID_GAME_STATE when this card is in no list).
//! * CheckPokemonAttacksEffect: when this card is in play for the effect's
//!   player and its Ability is not locked (phase 4b: the lock probe is new),
//!   an evolved Active Pokémon adds the attacks of every other Pokémon card in
//!   its slot. Phase 4b: it used to add them for every evolved in-play Pokémon
//!   (the Active could use the previous-Evolution attacks of Benched Pokémon,
//!   and a Basic Active those of a Benched evolved one). Each Relicanth in
//!   play still adds them again.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Relicanth@TEF",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::GrantAttacks(GrantAttacksSpec {}) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
