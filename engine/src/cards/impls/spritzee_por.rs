//! Spritzee (POR 35): Sweet Scent - heal 30 damage from 1 of your Pokémon. Ram - 10.
//!
//! A non-cancellable pick over your Active and Bench, then a RemoveCounters (heal) of 30 on the choice, caused by the attack
//! (Dolliv DRI's Nutrients).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SpritzeePOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me) }), hp: Num::Lit(30), clear_conditions: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
