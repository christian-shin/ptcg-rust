//! Hawlucha (POR 46): Vengeful Kick - 30+; if your Benched Pokémon have any damage counters on them, this attack does 60 more damage.
//!
//! Adds 60 before the damage when any of your Benched Pokémon is damaged (Mega Darkrai ex PBL 48).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Hawlucha",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(60, Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Damaged))),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
