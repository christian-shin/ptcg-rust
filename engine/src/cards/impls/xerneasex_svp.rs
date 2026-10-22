//! Xerneas ex (SVP 179): Aurora Beam - 50. Rising Horns - 120+; if your opponent's Active Pokémon is a Pokémon ex, this
//! attack does 100 more damage.
//!
//! Read before the damage from the Defending Pokémon (Bouffalant ex WHT 77's Gold Breaker).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "XerneasEx",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(more_damage_if(100, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
