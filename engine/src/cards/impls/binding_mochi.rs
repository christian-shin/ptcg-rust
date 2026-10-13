//! Binding Mochi (SFA, tool): attacks used by the Poisoned Pokémon this card is attached to do 40 more damage to your
//! opponent's Active Pokémon.
//!
//! A `DamageDealt` bonus at the Deal stage (before Weakness and Resistance) whose attacker is the Holder while it has the
//! Poisoned condition; it applies to damage dealt to the opponent's Active Pokémon only. A Tool-effects lock turns it off.
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
