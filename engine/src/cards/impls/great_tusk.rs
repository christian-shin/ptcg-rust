//! Great Tusk (TEF): Land Collapse — discard the top card of your opponent's
//! deck; if you played an Ancient Supporter from your hand this turn,
//! discard 3 more. Giant Tusk — 160.
//!
//! "If you played an Ancient Supporter card from your hand this turn": the played-this-turn record (APR E-26).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "GreatTusk",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(1)), ..DiscardSpec::DEFAULT })),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::PlayedThisTurn(Who::Me, Pred::All(&[Pred::Supporter, Pred::Tag(tag::ANCIENT)])),
                yes: &[Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(3)), ..DiscardSpec::DEFAULT }))],
                no: &[],
            }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
