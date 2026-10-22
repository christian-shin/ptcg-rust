//! Marnie's Purrloin (DRI 130): Pointy Nails - 20+; if your opponent's Active Pokémon is a Pokémon ex, this attack does 40 more damage.
//!
//! Read before the damage from the Defending Pokémon (Bouffalant ex WHT 77's Gold Breaker).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesPurrloin",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(40, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
