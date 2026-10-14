//! Dolliv (DRI): Nutrients — heal 40 damage from 1 of your Pokémon.
//! Tackle — 40.
//!
//! Rule: a non-cancellable pick over your Active and Bench (undamaged Pokémon
//! selectable, purpose DamageTarget), then a RemoveCounters (heal)
//! of 40 on the choice, caused by the attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dolliv@DRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), purpose: PromptPurpose::DamageTarget }), hp: Num::Lit(40), clear_conditions: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
