//! Entei (ASC 25): Flare Fall - 30+; if you have at least 4 [R] Energy in play, this attack does 90 more damage.
//!
//! The [R] Energy your Pokémon provide are counted before the damage, Basic or not (id2291: the text doesn't say Basic; a Prism
//! Energy counts, id483).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Entei",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(90, Cond::Cmp(Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(ct::FIRE)), CmpOp::Ge, Num::Lit(4))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
