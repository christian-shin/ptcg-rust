//! Mow Rotom (DRI 9): Trimming Mower - 20; discard a Stadium in play. Gadget Show - 30x; this attack does 30 damage for each
//! Pokémon Tool attached to all of your Pokémon.
//!
//! Trimming Mower discards the Stadium in play after the damage (Eternatus PFL's Shatter). Gadget Show counts the Tools on
//! your own Pokémon only (`Num::ToolCount(SlotSel::Pokemon(Who::Me))`; Bronzong PFL counts both sides) before the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MowRotomDRI",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(DISCARD_STADIUM)] },
        AttackSpec { index: 1, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::ToolCount(SlotSel::Pokemon(Who::Me)), &Num::Lit(30))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
