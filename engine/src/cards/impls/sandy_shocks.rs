//! Sandy Shocks (TEF): Magnetic Burst — 20+; 70 more if you have 3 or more
//! Energy in play; this attack's damage isn't affected by Weakness.
//! Power Gem — 60.
//!
//! Twinleaf sets `ignoreWeakness` first, then counts `provides` entries of
//! a CheckProvidedEnergyEffect per in-play Pokémon.
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
