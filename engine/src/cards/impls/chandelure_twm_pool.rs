//! Chandelure (TWM 38): Alluring Light - once during your turn, you may have each player draw a card. Mind Ruler - 30
//! damage for each card in your opponent's hand.
//!
//! The Ability's once-per-turn usage record is the `Once::PerTurn` entry; it is usable while either deck has a card (a Draw that
//! would draw nothing changes nothing). You draw first, then your opponent: two Draw events, each carrying its card (D3).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Chandelure",
    // Alluring Light: once during your turn, you may have each player draw a card.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(1)) })),
            Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(1)) })),
        ],
    }],
    // Mind Ruler: 30 damage for each card in your opponent's hand.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::ZoneSize(ZoneRef(Who::Opp, Zone::Hand)), &Num::Lit(30))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
