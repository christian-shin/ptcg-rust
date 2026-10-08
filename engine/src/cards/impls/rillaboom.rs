//! Rillaboom (TWM 16): Drum Beating — 60; during your opponent's next turn,
//! attacks used by the Defending Pokémon cost [C] more, and its Retreat Cost
//! is [C] more. Wood Hammer — 180; this Pokémon also does 50 damage to itself.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Rillaboom",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::IncreaseAttackCost })), Step::after_damage(Op::Arm(ArmSpec { what: Lasting::IncreaseRetreatCost }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(MY_ACTIVE), hp: Num::Lit(50), target_damage_mul: 0, calc: DamageCalc::Deal, when: Cond::True }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
