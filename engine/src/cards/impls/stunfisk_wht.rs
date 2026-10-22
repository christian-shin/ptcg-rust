//! Stunfisk (WHT 35): Muddy Bolt — 20+; if this Pokémon has any [F] Energy attached, this attack does 20 more damage. Flop — 50.
//!
//! The bonus applies when this Pokémon has any [F] Energy provided ("Energy attached" counts the Energy provided, id2237).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "StunfiskWHT",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(more_damage_if(20, Cond::Cmp(Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::Provided(ct::FIGHTING)), CmpOp::Gt, Num::Lit(0))))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
