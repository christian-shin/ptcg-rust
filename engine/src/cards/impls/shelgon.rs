//! Shelgon (JTG, support card): Guard Press - 30; during your opponent's next turn, this Pokémon takes 30 less damage from
//! attacks (after applying Weakness and Resistance). Heavy Impact - 80.
//!
//! Guard Press arms `Lasting::TakesLessDamage(30)` on this Pokémon (an ApplyEffect event), read in the damage calculation
//! after Weakness and Resistance until the end of the opponent's next turn.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shelgon@JTG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(30) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
