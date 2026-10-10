//! Dedenne (SSP): Electromagnetic Sonar - put a Trainer card from your discard
//! pile into your hand. Gnaw - 30.
//!
//! Twinleaf: ChooseCardsPrompt (max 1, no cancel) over the discard pile; the
//! chosen cards are shown to the opponent (info prompt only when any) and
//! moved. Fixed in phase 4b (R4): min is 1 when the discard pile holds a
//! Trainer card (it was always 0, so the Trainer could be declined).
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
