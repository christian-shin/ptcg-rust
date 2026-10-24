//! Erika's Victreebel (ASC 6): Flower Garden Rondo - 40x; this attack does 40 damage for each of your Erika's Pokémon in play. Solar
//! Beam - 130.
//!
//! The damage is 40 times the Pokémon in play (Active and Bench, this Pokémon included) whose name has "Erika's" (the Erika's tag),
//! set before the damage.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "ErikasVictreebel",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::InPlayCount(Who::Me, PlayScope::All, Pred::Tag(tag::ERIKAS)), &Num::Lit(40))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
