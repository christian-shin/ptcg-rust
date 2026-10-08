//! Crustle (DRI): Mysterious Stone House — prevent all damage done to this
//! Pokémon by attacks from your opponent's Pokémon ex. Great Scissors — 120;
//! this attack's damage isn't affected by any effects on your opponent's
//! Active Pokémon.
//!
//! Twinleaf: the ability only reacts to PutDamageEffect while this card is
//! the target's top Pokémon, during the attack phase; the lock check reduces
//! a real PowerEffect for Mysterious Stone House (skipped for a Shred attack's
//! damage). Great Scissors sets `ignoreDefenderEffects` (phase 4b R7B: it used
//! to add the damage straight to the Active, skipping the attacker's effects).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Crustle@DRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::PreventDamage(PreventDamageSpec {
            subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
            source: SlotPred::Tag(tag::POKEMON_EX_LOWER),
            ..PreventDamageSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
