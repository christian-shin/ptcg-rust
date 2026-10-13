//! Gladion's Final Battle (PBL 77, Supporter): you can use this card only when it is the last card in your hand. During this turn,
//! attacks used by your Pokémon that don't have a Rule Box do 80 more damage to your opponent's Active Pokémon (before
//! applying Weakness and Resistance).
//!
//! The last-card condition is the printed `Cond::Printed(LastCardInHand)` and only binds the play (JP FAQ Gladion's Final
//! Battle: drawing afterwards keeps the +80). Playing it puts an effect on the player for the turn (APR C-19):
//! the declared `Modifier::DamageDealt` (+80, attacker without a Rule Box), which ends with the turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "GladionsFinalBattle",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        // Usable only as the last card in your hand.
        needs: &[Cond::Printed(&Cond::LastCardInHand(Who::Me))],
        steps: &[Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Player(Who::Me), until: DeclareUntil::EndOfTurn }))],
    }),
    effects: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 80,
            attacker: SlotPred::Not(&SlotPred::RuleBox),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
