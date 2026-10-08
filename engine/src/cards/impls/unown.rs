//! Unown (30C): Mysterious Signal — 40; if your opponent's Pokémon is
//! Knocked Out by damage from this attack, take 1 more Prize card.
//!
//! Twinleaf: IF_OPPONENTS_POKEMON_KO_BY_ATTACK_DAMAGE_TAKE_MORE_PRIZES with
//! `attackName`; every Unown copy (any zone) reacts to the KnockOutEffect:
//! the target must be a Pokémon in the owner's Active/Bench, the phase must
//! be ATTACK with the attacker active, the owner carries DAMAGE_DEALT_MARKER
//! and the attacker's `playerLastAttack` is this card's Mysterious Signal.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Unown@30C",
    passives: &[Passive {
        origin: RuleSource::CardRule,
        // Mysterious Signal: if the opponent's Pokémon is Knocked Out by damage from this attack, take 1 more Prize card.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec { delta: 1, subject: SlotPred::Any, by_attack_damage: true, by_own_attack: Some("Mysterious Signal"), guard: Cond::True }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
