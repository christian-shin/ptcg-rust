//! Premium Power Pro (ASC 199, M1L Power Protein; Item): during this turn, attacks used by your [F] Pokémon do 30 more damage to
//! your opponent's Active Pokémon (before applying Weakness and Resistance).
//!
//! Playing it puts an effect on the player for the turn (APR C-19): a declared `DamageDealt` of +30 for the attacker's current
//! type [F] (`SlotPred::TypeIs`, the derived type: APR D-07), added to damage above 0 (one per copy played), ended with the turn.
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "PremiumPowerPro",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Player(Who::Me), until: DeclareUntil::EndOfTurn }))],
    }),
    effects: &[Passive {
        origin: RuleSource::TrainerEffect,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            amount: 30,
            attacker: SlotPred::TypeIs(ct::FIGHTING),
            needs_damage: true,
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
