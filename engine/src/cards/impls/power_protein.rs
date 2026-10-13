//! Premium Power Pro (ASC 199, M1L Power Protein; Item): during this turn, attacks used by your [F] Pokémon do 30 more damage to
//! your opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! Playing it sets a player marker; a `DamageDealt` of +30 for the attacker's current type [F] (`SlotPred::TypeIs`, the derived
//! type: APR D-07), added to damage above 0 while a marker of this card is set (one per copy
//! played), cleared at the end of your turn.
use crate::spec::prelude::*;
use crate::types::ct;

const MARKER: &str = "POWER_PROTEIN_MARKER";

pub static SPEC: CardSpec = CardSpec {
    class: "PowerProtein",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, source: RuleSource::TrainerEffect }))],
    }),
    passives: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 30,
            attacker: SlotPred::TypeIs(ct::FIGHTING),
            needs_damage: true,
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
