//! Lumiose Galette (POR 78, Item): heal 20 damage and remove a Special Condition from your Active Pokémon.
//!
//! Playable when the Active Pokémon has damage or a Special Condition (JP FAQ Lumiose Galette). One Heal event of 20, then
//! `ConditionChange::RemoveChosen`: the only condition is removed without a prompt, with several the player chooses which one.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "LumioseGalette",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::Any(&[Cond::Slot(MY_ACTIVE, SlotPred::Damaged), Cond::Slot(MY_ACTIVE, SlotPred::HasCondition)])],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(20), clear_conditions: false })),
            Step::new(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::RemoveChosen, when: Cond::True })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
