//! Toucannon (PBL): Aerial Draw — once during your turn, you may use this Ability. Draw a card. Feather Rondo — 60+; 20 more
//! damage for each Benched Pokémon, both yours and your opponent's.
//!
//! The Draw is one Draw event of the Ability's effect; with an empty deck it can't be used (id2231).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Toucannon",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) }))],
    }],
    attacks: &[AttackSpec {
        index: 0,
        // 20 more damage for each Benched Pokémon, both sides'.
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Add(&Num::BenchCount(Who::Me), &Num::BenchCount(Who::Opp)), &Num::Lit(20)), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
