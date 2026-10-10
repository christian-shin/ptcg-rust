//! Crustle (BCR 85): Sturdy — if this Pokémon has full HP and would be Knocked
//! Out by damage from an attack, it is not Knocked Out and its remaining HP
//! becomes 10 instead. Stone Edge — 70+; flip a coin, if heads 20 more damage.
//!
//! Rule: `SurviveOnTen(IfFullHp)` is read by the Damage event's calculation
//! (`damage::survive_on_10`): when the Pokémon has no damage counters and the
//! damage reaches its HP, it takes HP - 10 instead. It is an Ability, so an
//! Ability lock turns it off.
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
