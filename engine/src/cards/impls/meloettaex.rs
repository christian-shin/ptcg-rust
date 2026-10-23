//! Meloetta ex (BLK 44): Debut Performance — if you go first, this Pokémon can attack
//! on your first turn. Echoed Voice — 30; during your next turn, this
//! Pokémon's Echoed Voice attack does 80 more damage.
//!
//! Debut Performance is `Modifier::AttackFlags { first_turn }`: this Pokémon can attack on the first turn of the player who goes first,
//! while its Ability works. Echoed Voice's bonus is a `NextTurnBonus` of 80 to this Pokémon's Echoed Voice attack during your next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MeloettaEx",
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: Some(FirstTurnAttack::ThisPokemon), shred: None }) },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::NextTurnBonus(NextTurnBonusSpec { attack: 0, applies_to: BonusFor::Armer, change: BonusChange::More(80) }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
