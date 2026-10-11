//! Dedenne (SSP 87): Electromagnetic Sonar - put a Trainer card from your discard pile into your hand. Gnaw - 30.
//!
//! The Trainer card is chosen (one is required when the discard pile holds a Trainer card: no "up to"), revealed to the
//! opponent, and put into the hand by a PutIntoHand event from the discard pile (cause: this attack). Poké Vital A and
//! Neutralization Zone refuse that event for their own card (a lock over PutIntoHand).
use crate::spec::prelude::*;

const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);

pub static SPEC: CardSpec = CardSpec {
    class: "Dedenne",
    // Electromagnetic Sonar: put a Trainer card from your discard pile into your hand.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Pick(PickSpec {
                from: DISCARD,
                predicate: Pred::Trainer,
                bounds: Bounds { min: Num::If(&Cond::Nonempty(DISCARD, Pred::Trainer), &Num::Lit(1), &Num::Lit(0)), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::after_damage(Op::Reveal(RevealSpec { cards: RevealWhat::Chosen(0), by: Who::Me, to: Who::Opp, when_empty: false })),
            Step::after_damage(Op::PutIntoHand(PutIntoHandSpec { from: DISCARD, cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
