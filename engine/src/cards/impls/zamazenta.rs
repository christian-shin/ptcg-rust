//! Zamazenta (DRI 146): Strong Bash - 70; during your opponent's next turn, if this Pokémon is damaged by an attack (even if this
//! Pokémon is Knocked Out), put damage counters on the Attacking Pokémon equal to the damage done to this Pokémon.
//!
//! The lasting trigger of Mega Slowbro ex's Shellnado Spin, with as many counters as the damage the attack did to this Pokémon, as done:
//! 200 damage puts 20 counters even though Zamazenta has 130 HP (id2165). It is on this Pokémon, so retreating or evolving ends it
//! (APR C-15), and it resolves after the attack's own effects (APR E-03).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zamazenta",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Retaliate(PutOnAttacker::DamageDone) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
