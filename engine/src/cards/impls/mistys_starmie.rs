//! Misty's Starmie (DRI 47): Abrupt Flash - 60+; if this Pokémon evolved from Misty's Staryu during this turn, this attack does 80 more damage.
//!
//! The bonus reads the slot: its top card was played this turn and the card below it is Misty's Staryu (`PlayedThisTurn` and `CardBelowThis`, as Gholdengo's first attack).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MistysStarmie",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(80, Cond::Slot(SlotExpr::This, SlotPred::All(&[SlotPred::PlayedThisTurn, SlotPred::CardBelowThis("Misty's Staryu")])))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
