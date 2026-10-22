//! Raikou (MEG 48): Electro Fall - 30+; if you have at least 4 [L] Energy in play, this attack does 90 more damage.
//!
//! The [L] Energy your Pokémon provide are counted before the damage, Basic or not (id2291: the text doesn't say Basic; a Prism
//! Energy counts, id483).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Raikou",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(90, Cond::Cmp(Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::Provided(ct::LIGHTNING)), CmpOp::Ge, Num::Lit(4))))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
