//! Power Protein (M1L, item; Twinleaf name "Premium Power Pro"): during this
//! turn, your [F] Pokémon's attacks do 30 more damage to your opponent's
//! Active Pokémon (before applying Weakness and Resistance).
//!
//! Twinleaf: a per-instance player marker; every DealDamageEffect whose
//! player's Active is a (printed) [F] Pokémon, with damage > 0 and the
//! opponent's Active as target, gets +30 per marked copy (the discarded
//! card keeps reacting). Markers are removed at any EndTurn of their holder.
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
            attacker: SlotPred::PrintedTypeIs(ct::FIGHTING),
            needs_damage: true,
            guard: Cond::HasMarker { who: Who::Me, name: MARKER, from: MarkerFrom::This },
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: MARKER, from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
