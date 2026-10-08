//! Chandelure (TWM 38): Alluring Light — once during your turn, you may have
//! each player draw a card. Mind Ruler — 30 damage for each card in your
//! opponent's hand.
//!
//! Twinleaf: the marker is removed on PlayPokemon of this card and at the end
//! of each turn; the ability throws BLOCKED_BY_EFFECT when blocked,
//! POWER_ALREADY_USED with the marker, CANNOT_USE_POWER with both decks empty;
//! then marker + ABILITY_USED, you draw 1, then your opponent draws 1.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ChandelureTWMPool",
    // Alluring Light: once during your turn, you may have each player draw a card.
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("TWM_CHANDELURE_ALLURING_LIGHT"),
        needs: &[Cond::Not(&Cond::All(&[Cond::ZoneIs(ZoneRef(Who::Me, Zone::Deck), 0), Cond::ZoneIs(ZoneRef(Who::Opp, Zone::Deck), 0)]))],
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
