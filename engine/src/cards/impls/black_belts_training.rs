//! Black Belt's Training (PRE / JTG): during this turn, attacks used by your
//! Pokémon do 40 more damage to your opponent's Active Pokémon ex (before
//! Weakness and Resistance).
//!
//! Twinleaf: the card moves itself to the supporter pile and marks the
//! player; each copy only honors its own marker. The bonus applies to any
//! DealDamageEffect of a marked player whose target is the opponent's Active
//! Pokémon ex (phase 4b: it also applied to the player's own Active Pokémon ex,
//! e.g. recoil damage to itself).
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
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: TRAINING, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
