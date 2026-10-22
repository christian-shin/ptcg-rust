//! Abomasnow (DRI 60): Lunge Out — 90. Frozen Wood — 120+; if this Pokémon has 2 or more [G] Energy attached, this attack does 120
//! more damage.
//!
//! The bonus applies when the Energy this Pokémon provides includes 2 or more [G] ("Energy attached" counts the Energy provided,
//! id2237).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AbomasnowDRI",
    attacks: &[AttackSpec {
        index: 1,
        steps: &[Step::before_damage(more_damage_if(120, Cond::Cmp(Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::GRASS)), CmpOp::Ge, Num::Lit(2))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
