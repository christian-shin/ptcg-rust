//! Team Rocket's Blipbug (DRI 15): Searching Eyes - look at 1 of your opponent's face-down Prize cards.
//!
//! With a face-down Prize card of the opponent's (a face-up one isn't offered), you choose one of them and look at it: it is shown to
//! you only and stays face down (Bother-Bot turns the card face up instead). Nothing happens when every Prize card of the opponent
//! is face up or taken.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsBlipbug",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::FaceDownPrize(Who::Opp),
            yes: &[
                Step::new(Op::PickPrize(PickPrizeSpec { chooser: Who::Me, whose: Who::Opp, face_down_only: true, purpose: PromptPurpose::LookAtPrize })),
                Step::new(Op::PrizeVisibility(PrizeVisibilitySpec { action: PrizeAction::Look(Who::Opp) })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
