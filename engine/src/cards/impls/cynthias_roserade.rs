//! Cynthia's Roserade (DRI): Glorious Cheer - attacks from your Cynthia's
//! Pokémon deal 30 more damage to your opponent's Active Pokémon.
//!
//! Twinleaf: every DealDamageEffect of the player with this card in play gets
//! +30 when the source slot's Pokémon has the Cynthia's tag.
//!
//! Fixed (phase 4b, W4): the IS_ABILITY_BLOCKED probe ran but its result was
//! ignored, and the bonus applied to any target; it now stops when the
//! Ability is blocked and only applies to the opponent's Active Pokémon.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "CynthiasRoserade",
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 30, attacker: SlotPred::Tag(tag::CYNTHIAS), ..DamageDealtSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
