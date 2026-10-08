//! Enamorus (TWM): Heart Sign - 30. Love Resonance - 80+; 120 more if any of
//! your Pokémon in play share a type with any of your opponent's.
//!
//! Twinleaf reads the types of `cardList.getPokemonCard()` (the top Pokémon of
//! each in-play slot; phase 4b: it used to read `cardList.cards[0]`, the
//! bottom Pokémon of an evolved stack).
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
