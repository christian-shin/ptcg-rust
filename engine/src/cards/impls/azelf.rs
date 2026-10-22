//! Azelf (SSP 80): Neurokinesis - 10+; this attack does 10 more damage for each damage counter on all of your opponent's Pokémon.
//!
//! The damage (HP) on every Pokémon of the opponent, Active and Bench, is added before the damage: 10 per damage counter
//! (Zapdos SVP 157 counts the Defending Pokémon alone).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Azelf",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::DamageSum(SlotSel::Pokemon(Who::Opp), SlotPred::Any), when: Cond::True }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
