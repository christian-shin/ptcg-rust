//! Amarys (PRE 93, Supporter): draw 4 cards. At the end of this turn, if you have 5 or more cards in your hand, discard your hand.
//!
//! Draw 4 is one Draw event (cause: this Supporter). The end-of-turn clause is a trigger on the EndTurn event of the owner's turn
//! (APR E-21: "at the end of this turn" effects, ordered with the other end-of-turn effects by the player whose turn it is, id2135),
//! held by the Supporter that was played this turn (`RuleSource::TrainerEffect`: the owner's record of the cards played this turn,
//! though the card is in the discard pile by then): another copy in the hand, the deck or the discard pile has no such effect, so
//! it is discarded with the rest of the hand. The trigger discards the whole hand
//! when it has 5 or more cards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Amarys",
    play: Some(PlaySpec { kind: PlayKind::Supporter, needs: &[], steps: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(4)) }))] }),
    triggers: &[Trigger {
        origin: RuleSource::TrainerEffect,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Ge, Num::Lit(5)),
            yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, into: None }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
