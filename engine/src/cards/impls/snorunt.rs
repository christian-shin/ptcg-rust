//! Snorunt (TWM): Astonish — 20; choose a random card from your opponent's hand. Your opponent reveals that card and shuffles it
//! into their deck.
//!
//! The random card goes from the opponent's hand into their deck, revealed to you (a PutIntoDeck caused by the attack), and
//! their deck is shuffled (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Snorunt",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Hand), Pred::Any),
                yes: &[
                    Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), position: DeckPosition::Bottom, reveal: Some(Who::Me), ..PutIntoDeckSpec::DEFAULT })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
