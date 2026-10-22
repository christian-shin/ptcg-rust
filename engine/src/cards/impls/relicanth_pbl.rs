//! Relicanth (PBL 17): Fossil Beatdown — 10+; this attack does 30 more damage for each of your Benched Pokémon that has "Antique"
//! in its name.
//!
//! A before-damage `DamageOp::Add` of 30 times the Benched Pokémon whose top card's name contains "Antique" (the Antique Fossils
//! are Pokémon in play; "in its name" is the card's name, as for "Tauros", id2255).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RelicanthPBL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Damage(DamageSpec {
            op: DamageOp::Add,
            hp: Num::Mul(&Num::SlotCount(SlotSel::Bench(Who::Me), SlotPred::Top(Pred::NameContains("Antique"))), &Num::Lit(30)),
            when: Cond::True,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
