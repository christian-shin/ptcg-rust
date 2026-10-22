//! Marnie's Liepard (DRI 131): Pointy Claws - 70+; if your opponent's Active Pokémon is a Pokémon ex, this attack does 70 more damage.
//!
//! `more_damage_if` on the top card of the opponent's Active (Marnie's Purrloin DRI 130).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MarniesLiepard",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(70, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx)))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
