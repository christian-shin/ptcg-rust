//! Clefable (TWM / ASC): Metronome - choose 1 of your opponent's Active
//! Pokémon's attacks and use it as this attack (COPY_OPPONENT_ACTIVE_ATTACK_WITH_RETRY,
//! see `copy_attack.rs`). Magical Shot - 100.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Clefable@TWM|ASC",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::CopyAttack(CopyAttackSpec { from: Who::Opp, predicate: Pred::Any, retries: 3, scope: CopyScope::Active })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
