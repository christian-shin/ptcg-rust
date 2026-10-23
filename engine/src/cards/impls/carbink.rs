//! Carbink (ASC 117): Double Type - as long as this Pokémon is in play, it is [F] and [P] type. Counter Jewel - 70+; if your
//! opponent has 2 or fewer Prize cards remaining, this attack does 100 more damage.
//!
//! Double Type is a `TypeOverride` on this Pokémon (the derived type read, so every "[F] Pokémon" / "[P] Pokémon" read of the
//! game sees both types; Iron Treads TEF 118), while its Ability works. Counter Jewel adds 100 before the damage when the
//! opponent has 2 or fewer Prize cards left.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Carbink",
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::TypeOverride(TypeOverrideSpec { subject: SlotPred::IsThisPokemon, set: &[ct::FIGHTING, ct::PSYCHIC] }) }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(100, Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(2))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
