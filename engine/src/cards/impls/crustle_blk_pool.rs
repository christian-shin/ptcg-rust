//! Crustle (BLK): Sturdy - if this Pokémon has full HP and would be Knocked
//! Out by damage from an attack, it is not Knocked Out and its remaining HP
//! becomes 10 (SURVIVE_ON_TEN_IF_FULL_HP, see `crustle_bcr.rs`). Stone Edge -
//! 80+; flip a coin, if heads 60 more damage.
//!
//! Twinleaf fix (phase 4b, Y2-1): Sturdy's reason is the literal 'Sturdy'; it
//! read `this.powers[0].name`, which threw for a copycat without Abilities
//! (Zoroark's Foul Play, Ethan's Sudowoodo's Try to Imitate) on the first
//! effect its copy session delegated to this code.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CrustleBLKPool",
    // Sturdy: if this Pokémon has full HP and would be Knocked Out by damage from an attack, it is
    // not Knocked Out and its remaining HP becomes 10.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::SurviveOnTen(SurviveOnTenSpec { how: SurviveHow::FullHp }) }],
    // Stone Edge: flip a coin, if heads 60 more damage.
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Coin(CoinSpec { heads: &[Step::new(more_damage_if(60, Cond::True))], ..CoinSpec::DEFAULT }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
