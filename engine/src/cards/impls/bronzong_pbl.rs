//! Bronzong (PBL 64): Gentle Slap — 40. Metal Block — 120; during your opponent's next turn, this Pokémon takes 100 less damage
//! from attacks from Evolution Pokémon (after applying Weakness and Resistance).
//!
//! Metal Block is one ApplyEffect event on this Pokémon: a lasting "takes less damage" for the attacks of Pokémon that are
//! Evolution Pokémon (the attacker read where it is when the damage is done, `DamageSource::Evolution`), taken off at step 5 after
//! Weakness and Resistance (APR B-05), ending with the opponent's next turn or when this Pokémon leaves the Active Spot (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BronzongPBL",
    attacks: &[AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(100, DamageSource::Evolution) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
