//! Metapod (JTG 2): Harden — during your opponent's next turn, prevent all damage done to this Pokémon by attacks if that damage
//! is 60 or less.
//!
//! The same text as Roggenrola WHT 46's Harden with a bound of 60: the damage after Weakness, Resistance and the effects on this
//! Pokémon (id472, id2197), read where a prevention is read (step 6 of the damage calculation); gone when this Pokémon leaves the
//! Active Spot (id2196). An effect the attack puts on the Pokémon (an ApplyEffect event).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MetapodJTG",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamageAtMost(60) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
