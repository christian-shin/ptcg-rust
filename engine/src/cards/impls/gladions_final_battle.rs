//! Gladion's Final Battle (PBL 77, Supporter): you can use this card only when it is the last card in your hand. During this turn,
//! attacks used by your Pokémon that don't have a Rule Box do 80 more damage to your opponent's Active Pokémon (before
//! applying Weakness and Resistance).
//!
//! The last-card condition is the printed `Cond::Printed(LastCardInHand)` and only binds the play (JP FAQ Gladion's Final
//! Battle: drawing afterwards keeps the +80). Playing it sets a player marker; while the marker is on, a
//! `Modifier::DamageDealt` (+80, attacker without a Rule Box) applies, and an EndTurn trigger clears it.
use crate::spec::prelude::*;

const MARKER: &str = "M5_GLADIONS_DECISIVE_BATTLE";

pub static SPEC: CardSpec = CardSpec {
    class: "GladionsFinalBattle",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        // Usable only as the last card in your hand.
        needs: &[Cond::Printed(&Cond::LastCardInHand(Who::Me))],
        steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, source: RuleSource::TrainerEffect }))],
    }),
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 80,
            attacker: SlotPred::Not(&SlotPred::RuleBox),
            guard: Cond::HasMarker { who: Who::Me, name: MARKER, from: MarkerFrom::This },
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
