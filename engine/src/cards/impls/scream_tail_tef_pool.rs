//! Scream Tail (TEF 77): Supportive Singing - heal 100 damage from 1 of your Benched Ancient Pokémon. Hyper Voice - 40.
//!
//! A Heal of 100 on a Benched Pokémon whose top card is an Ancient Pokémon (the `ANCIENT` tag), chosen by the player; nothing
//! happens with none.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "ScreamTailTEFPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Tag(tag::ANCIENT))), }), hp: Num::Lit(100), clear_conditions: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
