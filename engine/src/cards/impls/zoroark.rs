//! Zoroark (SV11W / WHT 62): Mind Jack — 30 damage for each of your
//! opponent's Benched Pokémon. Foul Play — choose 1 of your opponent's Active
//! Pokémon's attacks and use it as this attack (COPY_OPPONENT_ACTIVE_ATTACK,
//! see `copy_attack.rs`).
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
