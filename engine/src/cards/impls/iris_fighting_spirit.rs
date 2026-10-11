//! Iris's Fighting Spirit (ASC 190): Supporter. You can use this card only if you discard another card from your hand.
//! Draw cards until you have 6 cards in your hand.
//!
//! The discard is the cost (`Step::cost`, APR E-30): one Discard event of the chosen card from the hand (cause: this
//! Supporter); another card is required (`Cond::Printed`: it still binds a copy used through Look-Alike Show, D5), and
//! the Supporter itself isn't in the hand when the card is chosen. Then one Draw event of the cards missing from 6. The
//! other `needs` entries (a card to draw: fewer than 7 other cards in hand, a card in the deck) restate that a Trainer
//! that can't have any effect can't be played (id925, id1124) and go at the D10 switch.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IrisFightingSpirit",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Ge, Num::Lit(1))), Cond::Cmp(Num::OthersCount(ZoneRef(Who::Me, Zone::Hand), Pred::Any), CmpOp::Lt, Num::Lit(7))],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::If(IfSpec { cond: Cond::True, yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))], no: &[] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
