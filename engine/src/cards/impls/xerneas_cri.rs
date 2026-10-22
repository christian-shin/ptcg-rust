//! Xerneas (CRI 42): Geo Storm - 30x; this attack does 30 damage for each [P] Energy attached to all of your Pokémon.
//!
//! The damage is 30 times the [P] Energy provided by every Pokémon of yours (an Energy that provides [P] or any type counts,
//! id2237; Hydrapple ex SCR 14).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "XerneasCRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(ct::PSYCHIC)), &Num::Lit(30)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
