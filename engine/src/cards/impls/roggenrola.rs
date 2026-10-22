//! Roggenrola (WHT 46): Harden - during your opponent's next turn, prevent all damage done to this Pokémon by attacks if that damage
//! is 40 or less. Rolling Rocks - 30.
//!
//! The lasting `Prevent` over the Damage of the opponent's attacks, only when the damage, after Weakness and Resistance and the
//! other effects on this Pokémon, is 40 or less (`EventPred::AmountAtMost`, id472, id2197: at step 6 of the damage calculation). It is an
//! effect on this Pokémon: it ends when it leaves the Active Spot (id2196).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Roggenrola",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventDamageAtMost(DamageCap::AtMost40) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
