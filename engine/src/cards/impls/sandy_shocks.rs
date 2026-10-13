//! Sandy Shocks (TEF 98): Magnetic Burst - 20+; if you have 3 or more Energy in play, this attack does 70 more damage, and this
//! attack's damage isn't affected by Weakness. Power Gem - 60.
//!
//! The attack sets the `NoWeakness` flag and adds 70 when the Energy provided by your Pokémon in play is 3 or more (units, not
//! cards: id2237, id2347; JP FAQ Sandy Shocks).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SandyShocks",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(more_damage_if(70, Cond::Cmp(Num::EnergyOn(SlotSel::Pokemon(Who::Me), EnergyUnit::ProvidedUnits), CmpOp::Ge, Num::Lit(3)))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
