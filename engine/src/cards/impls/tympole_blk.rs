//! Tympole (BLK 19): Round - 20x; this attack does 20 damage for each of your Pokémon in play that has the Round attack.
//!
//! The Pokémon of yours (Active and Bench, this one included) whose top card prints an attack named Round are counted before the
//! damage (Seismitoad BLK 21 prints Round too).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TympoleBLK",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(damage_is(Num::Mul(&Num::SlotCount(SlotSel::Pokemon(Who::Me), SlotPred::Top(Pred::HasAttackNamed("Round"))), &Num::Lit(20))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
