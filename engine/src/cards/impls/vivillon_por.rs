//! Vivillon (POR): Grand Wing — once during your turn, your opponent shuffles their hand and puts it on the bottom of their
//! deck. If they put any cards on the bottom of their deck in this way, they draw 4 cards. Blow Through — 60+; 60 more damage if a Stadium is in play.
//!
//! The hand goes to the bottom of the opponent's deck shuffled (a PutIntoDeck of the Ability's effect); "if they put any cards" is
//! `Cond::Done` on it (APR E-20), then the opponent draws 4 (a Draw event). It needs a card in the opponent's hand (id2306).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Vivillon@POR",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("BIG_WINGS_MARKER"),
        needs: &[],
        // Your opponent shuffles their hand and puts it on the bottom of their deck; if they did, they draw 4 cards.
        steps: &[
            Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), position: DeckPosition::Bottom, order: DeckOrder::Shuffled, ..PutIntoDeckSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::Done, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Opp, amount: DrawAmount::Count(Num::Lit(4)) }))], no: &[] })),
        ],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(60, Cond::StadiumInPlay(Pred::Any)))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
