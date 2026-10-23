//! Scovillain ex (SSP 37): Double Type - as long as this Pokémon is in play, it is [G] and [R] type. Spicy Rage - 10+; this
//! attack does 70 more damage for each damage counter on this Pokémon.
//!
//! Double Type is a `TypeOverride` on this Pokémon (Iron Treads TEF 118's Dual Core): the derived type read gives [G] and
//! [R], so every type read of the game does (id21: another type-adding effect adds to it, id1867: it counts as two types).
//! Spicy Rage adds 70 per damage counter: 7 times the HP of damage on this Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ScovillainEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::DamageOn(MY_ACTIVE), &Num::Lit(7)), when: Cond::True }))],
    }],
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::TypeOverride(TypeOverrideSpec { subject: SlotPred::IsThisPokemon, set: &[ct::GRASS, ct::FIRE] }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
