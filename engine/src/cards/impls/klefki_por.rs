//! Klefki (POR 59): Memory Lock — 30; choose 1 of your opponent's Active Pokémon's attacks. During your opponent's next turn, that
//! Pokémon can't use that attack.
//!
//! `PickAttack`: the chosen attack of the Defending Pokémon can't be used during the opponent's next turn; the lock is an effect on
//! that Pokémon and ends when it goes to the Bench, so retreating and coming back frees the attack (id2314).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "KlefkiPOR",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::PickAttack(PickAttackSpec { from: Who::Opp }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
