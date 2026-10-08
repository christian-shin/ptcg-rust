//! Toucannon (M5): Aerial Draw — once during your turn, draw a card.
//! Feather Rondo — 60+, 20 more for each Benched Pokémon (both sides).
//!
//! Twinleaf: IS_ABILITY_BLOCKED → BLOCKED_BY_EFFECT, then
//! USE_ABILITY_ONCE_PER_TURN, then an empty deck throws CANNOT_USE_POWER.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Toucannon",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("M5_TOUCANNON_SKYDRAW"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
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
