//! Heat Rotom (DRI 43): Singe - your opponent's Active Pokémon is now Burned. Gadget Show - 30x; this attack does 30 damage
//! for each Pokémon Tool attached to all of your Pokémon.
//!
//! Singe: a GainCondition(Burned) with the attack's cause. Gadget Show: the damage is 30 times the Tools attached to every
//! Pokémon of yours (`Num::ToolCount(SlotSel::Pokemon(Who::Me))`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HeatRotom",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(inflict(&[SpecialCondition::Burned])),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(damage_is(Num::Mul(&Num::ToolCount(SlotSel::Pokemon(Who::Me)), &Num::Lit(30)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
