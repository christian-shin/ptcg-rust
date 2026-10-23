//! Camerupt (ASC 28): Roasting Burn - 110; if your opponent's Active Pokémon isn't Burned, this attack does nothing. Power Stomp -
//! 170; discard 2 Energy from this Pokémon.
//!
//! Roasting Burn sets the damage to 0 before the damage unless the Defending Pokémon is Burned (an attack that does nothing is still
//! used, APR D-01, id675; Basculin WHT 24, Mesprit SSP 79). Power Stomp discards 2 Energy of any type from this Pokémon after the
//! damage (Honchkrow PFL 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Camerupt",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Set, hp: Num::Lit(0), when: Cond::Not(&Cond::Slot(OPP_ACTIVE, SlotPred::Condition(SpecialCondition::Burned))) }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
