//! Scream Tail (TEF 77): Supportive Singing — heal 100 damage from 1 of your
//! Benched Ancient Pokémon. Hyper Voice — 40.
//!
//! Twinleaf: nothing without a Benched Ancient Pokémon; else a mandatory
//! ChoosePokemonPrompt on the Bench with every non-Ancient (or empty) Bench
//! index blocked, and a HealEffect per chosen target.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "ScreamTailTEFPool",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Me), SlotPred::Top(Pred::Tag(tag::ANCIENT))), msg: "CHOOSE_POKEMON_TO_HEAL" }), hp: Num::Lit(100), clear_conditions: false })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
