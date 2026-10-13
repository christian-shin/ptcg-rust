//! Team Rocket's Mimikyu (ASC 97): Gemstone Mimicry — choose 1 of your opponent's Active Tera Pokémon's attacks and use it as this
//! attack.
//!
//! `Op::CopyAttack` over the opponent's Active Pokémon, offered only while it is a Tera Pokémon (retries 1, no cancel; see
//! `copy_attack.rs`): otherwise nothing happens. The copied attack is this Pokémon's own attack while it is used (id2402, id2094).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMimikyu",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::CopyAttack(CopyAttackSpec { from: Who::Opp, predicate: Pred::Tag(crate::types::tag::POKEMON_TERA), retries: 1, scope: CopyScope::Active })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
