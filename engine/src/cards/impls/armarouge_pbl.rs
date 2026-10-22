//! Armarouge (PBL 12): Flame Legion — 40+; this attack does 40 more damage for each of your Benched Pokémon that has any [R]
//! Energy attached.
//!
//! "More damage" is part of the damage calculation (APR B-01), so it is a before-damage `DamageOp::Add` of 40 times the Benched
//! Pokémon that provide [R] (Energy attached counts the Energy provided, id2237: a Prism Energy on a Basic Pokémon counts).
//! The class is pinned to PBL: Armarouge SSP 34 is another card.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ArmarougePBL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec {
            op: DamageOp::Add,
            hp: Num::Mul(&Num::SlotCount(SlotSel::Bench(Who::Me), SlotPred::Provides(ct::FIRE)), &Num::Lit(40)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
