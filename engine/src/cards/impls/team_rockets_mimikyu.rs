//! Team Rocket's Mimikyu (DRI 87 / ASC): Gemstone Hunt — choose an attack on
//! your opponent's Active Tera Pokémon and use it as the effect of this
//! attack.
//!
//! Twinleaf: nothing unless the opponent's Active has a Pokémon card with
//! attacks and the Tera tag; then COPY_ATTACK_FROM_POKEMON_LIST with that
//! card (allowCancel false, maxRetries 1, see `copy_attack.rs`).
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
