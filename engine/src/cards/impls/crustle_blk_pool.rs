//! Crustle (BLK): Sturdy - if this Pokémon has full HP and would be Knocked Out by damage from an attack, it is not
//! Knocked Out and its remaining HP becomes 10 (`SurviveOnTen`, see `crustle_bcr.rs`: a replacement inside the Damage
//! event's calculation). Stone Edge - 80+; flip a coin, if heads 60 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CrustleBLKPool",
    // Sturdy: if this Pokémon has full HP and would be Knocked Out by damage from an attack, it is
    // not Knocked Out and its remaining HP becomes 10.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { kind: SurviveKind::IfFullHp }) }],
    // Stone Edge: flip a coin, if heads 60 more damage.
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(60, Cond::True))], ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
