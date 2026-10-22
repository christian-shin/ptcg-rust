//! Suicune (PFL 26): Crystal Fall - 30+; if you have at least 4 [W] Energy in play, this attack does 90 more damage.
//!
//! The [W] Energy provided by all your Pokémon is counted as the game checks it (Provided units, id2237) before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Suicune",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(more_damage_if(90, Cond::Cmp(Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(ct::WATER)), CmpOp::Ge, Num::Lit(4))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
