//! Binding Mochi (SFA, tool): attacks used by the Poisoned Pokémon this card
//! is attached to do 40 more damage to your opponent's Active Pokémon.
//!
//! Twinleaf quirks kept: the Poisoned check reads the attacking player's
//! Active (not the tool's holder); the tool block probe is a bare ToolEffect
//! (no `stadiumAndToolHaveNoEffectTurnsRemaining` check).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BindingMochi",
    // Attacks used by the Poisoned Pokémon this card is attached to do 40 more damage to your
    // opponent's Active Pokémon.
    passives: &[Passive {
        origin: RuleSource::Tool,
        modifier: Modifier::DamageDealt(DamageDealtSpec {
            stage: DamageStage::Deal,
            amount: 40,
            attacker: SlotPred::All(&[SlotPred::Holder, SlotPred::Condition(SpecialCondition::Poisoned)]),
            ..DamageDealtSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
