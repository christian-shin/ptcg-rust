//! Clefable (TWM / ASC): Metronome - choose 1 of your opponent's Active
//! Pokémon's attacks and use it as this attack (see `copy_attack.rs`): the player chooses once; an attack that can't be fully done
//! is still used and ends. Magical Shot - 100.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Clefable",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::CopyAttack(CopyAttackSpec { from: Who::Opp, predicate: Pred::Any, scope: CopyScope::Active })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
