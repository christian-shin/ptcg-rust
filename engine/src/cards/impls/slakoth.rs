//! Slakoth (SSP): Take It Easy — heal 60 damage from this Pokémon. During
//! your next turn, this Pokémon can't retreat.
//!
//! Rule: Take It Easy is a RemoveCounters (heal) of 60 on this Pokémon, caused
//! by the attack, then a self retreat block whose target is the attacker (so
//! Mist Energy on the Defending Pokémon doesn't block it).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Slakoth",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(heal_active(60)), Step::after_damage(Op::Arm(ArmSpec { what: Lasting::SelfCannotRetreat }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
