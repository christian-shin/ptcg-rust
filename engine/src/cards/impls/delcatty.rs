//! Delcatty (ASC 166): Cat Kick - 40. Energy Crush - 40×; this attack does 40 damage for each Energy attached to all of your opponent's Pokémon.
//!
//! The damage is 40 times the Energy units provided across the opponent's Pokémon in play (Rotom SSP 61 counts Energy units; Diancie counts only Special Energy cards).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Delcatty",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Opp), EnergyUnit::ProvidedUnits), &Num::Lit(40)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
