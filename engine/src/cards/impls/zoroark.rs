//! Zoroark (WHT 62): Mind Jack — 30× for each of your opponent's Benched Pokémon. Foul Play — choose 1 of your opponent's Active Pokémon's
//! attacks and use it as this attack.
//!
//! Foul Play is `Op::CopyAttack` over the opponent's Active Pokémon (see `copy_attack.rs`): the copied attack is Zoroark's own while it is
//! used, so only the printed attack is copied, nothing of the source card's Abilities, and no Energy is needed for it (JP FAQ; id2402).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zoroark",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::BenchCount(Who::Opp), &Num::Lit(30))))] },
        // Foul Play: choose 1 of your opponent's Active Pokémon's attacks and use it as this attack.
        AttackSpec { index: 1, steps: &[Step::before_damage(Op::CopyAttack(CopyAttackSpec { from: Who::Opp, predicate: Pred::Any, retries: 1 , scope: CopyScope::Active, }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
