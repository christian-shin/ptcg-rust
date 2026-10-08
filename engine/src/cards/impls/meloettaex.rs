//! Meloetta ex (SV11B): Live Debut — if you go first, this Pokémon can attack
//! on your first turn. Echoed Voice — 30; during your next turn, this
//! Pokémon's Echoed Voice attack does 80 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Meloettaex",
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: true, shred: None }) },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::NextTurnBonus(NextTurnBonusSpec { attack: "Echoed Voice", bonus: 80 }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
