//! Lumiose Galette (POR): heal 20 damage and remove a Special Condition from
//! your Active Pokémon.
//!
//! Twinleaf: playable only if the Active has damage or a Special Condition;
//! a HealEffect for 20, then a Special Condition is removed directly.
//! Fixed (phase 4b, R3): the player chooses which one: the only condition is
//! removed without a prompt; with several, a SelectOptionPrompt lists all five
//! (Paralyzed, Confused, Asleep, Poisoned, Burned; absent ones disabled, not
//! cancellable) and the chosen one is removed (it used to remove the first
//! one listed).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LumioseGalette",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Any(&[Cond::Slot(MY_ACTIVE, SlotPred::Damaged), Cond::Slot(MY_ACTIVE, SlotPred::HasCondition)])],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(20), via: HealVia::Effect, clear_conditions: false })),
            Step::new(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::RemoveChosen, cause: Cause::Direct, gate: Gate::None, when: Cond::True })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
