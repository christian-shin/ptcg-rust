//! Sableye (PFL 59): Cocky Claw — 20+; if you have any Stage 2 [D] Pokémon on your Bench, this attack does 70 more damage.
//!
//! The bonus is added once when a Benched Pokémon is a Stage 2 card and is [D] as the game checks its type now (`SlotPred::TypeIs`,
//! APR D-07); the Active Pokémon, this one, doesn't count.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SableyePFL",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(more_damage_if(70, Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::All(&[SlotPred::Top(Pred::StageIs(crate::types::Stage::Stage2)), SlotPred::TypeIs(ct::DARK)]))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
