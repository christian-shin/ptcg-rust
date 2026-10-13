//! Enamorus (TWM 93): Heart Sign — 30. Love Resonance — 80+; 120 more damage if any of your Pokémon in play are the same type as
//! any of your opponent's Pokémon in play.
//!
//! `Cond::TypesShared` over both sides' Pokémon in play, Active and Bench, reads each Pokémon's types after effects
//! (`derived::types`: the top Pokémon card of an evolved stack, a type override counted), never the printed types.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Enamorus",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Lit(120), when: Cond::TypesShared(SlotSel::Pokemon(Who::Me), SlotSel::Pokemon(Who::Opp)) })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
