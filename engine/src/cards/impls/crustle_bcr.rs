//! Crustle (BCR): Sturdy — if this Pokémon has full HP and would be Knocked
//! Out by damage from an attack, it is not Knocked Out and its remaining HP
//! becomes 10. Stone Edge — 70+; flip a coin, if heads 20 more damage.
//!
//! SURVIVE_ON_TEN_IF_FULL_HP: on any PutDamageEffect whose target slot holds
//! this card (not necessarily on top), unless the ability is blocked for the
//! slot's owner, when the slot has no damage and `effect.damage >=` its HP
//! (CheckHpEffect by the owner), sets `surviveOnTenHPReason`. The core then
//! caps the damage at HP - 10 when it reached HP (phase 4b: exactly lethal
//! damage used to Knock Out anyway).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Crustle@BCR",
    // Sturdy: if this Pokémon has full HP and would be Knocked Out by damage from an attack, it is
    // not Knocked Out and its remaining HP becomes 10.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { kind: SurviveKind::IfFullHp }) }],
    // Stone Edge: flip a coin, if heads 20 more damage.
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(20, Cond::True))], ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
