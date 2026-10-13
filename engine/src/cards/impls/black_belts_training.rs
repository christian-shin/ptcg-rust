//! Black Belt's Training (PRE / JTG): during this turn, attacks used by your Pokémon do 40 more damage to your opponent's
//! Active Pokémon ex (before Weakness and Resistance).
//!
//! The Supporter marks the player (a Trainer effect); a `DamageDealt` bonus at the Deal stage reads the marker and applies to
//! damage dealt to the opponent's Active Pokémon ex only (never to your own, e.g. recoil), and only when the damage is above 0.
//! The marker ends with the turn (the player's EndTurn trigger).
use crate::spec::prelude::*;

const TRAINING: &str = "BLACK_BELTS_TRAINING_MARKER";

pub static SPEC: CardSpec = CardSpec {
    class: "BlackBeltsTraining",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: TRAINING, source: RuleSource::TrainerEffect }))],
    }),
    // During this turn, attacks used by your Pokémon do 40 more damage to your opponent's Active
    // Pokémon ex (before Weakness and Resistance).
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            stage: DamageStage::Deal,
            amount: 40,
            target: SlotPred::Tag(crate::types::tag::POKEMON_EX_LOWER),
            needs_damage: true,
            guard: Cond::HasMarker { who: Who::Me, name: TRAINING, from: MarkerFrom::This },
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    triggers: &[Trigger {
        origin: RuleSource::TrainerEffect,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EndTurn), EventPred::Owner(Who::Me)])),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: TRAINING, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
