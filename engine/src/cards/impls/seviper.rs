//! Seviper (M2 / PFL 62): Excite Power — if you have a [D] Mega Evolution
//! Pokémon ex in play, this Pokémon's attacks do 120 more damage. Jet Black
//! Fang — 120.
//!
//! Fixed (phase 4b, W4): Twinleaf accepted any [D] Pokémon ex; the Mega ex tag
//! is now required.
//!
//! Twinleaf: any AttackEffect whose source slot holds this card gets +120
//! when its damage is above 0 (checked after the ability-lock probe).
use crate::spec::prelude::*;
use crate::types::{ct, tag};

pub static SPEC: CardSpec = CardSpec {
    class: "Seviper",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            stage: DamageStage::Attack,
            amount: 120,
            attacker: SlotPred::Holder,
            side: Side::Any,
            needs_damage: true,
            // A [D] Mega Evolution Pokémon ex in play.
            guard: Cond::AnySlot(Who::Me, SlotPred::All(&[SlotPred::PrintedTypeIs(ct::DARK), SlotPred::Tag(tag::POKEMON_EX_LOWER), SlotPred::Tag(tag::POKEMON_SV_MEGA)])),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
