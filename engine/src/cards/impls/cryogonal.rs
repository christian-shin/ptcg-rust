//! Cryogonal (BLK 27): Drag Off - switch in 1 of your opponent's Benched Pokémon to the Active Spot. This attack does 20 damage
//! to the new Active Pokémon. Icicle - 30.
//!
//! Drag Off: a SwitchIn the attacker chooses (Arven's Toedscruel DRI 110), then 20 damage to the Active Pokémon of the opponent as it
//! is now, with Weakness and Resistance (an Active Spot); with no Benched Pokémon there is no switch and the Defending Pokémon
//! takes the damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cryogonal",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, required: false })),
            Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Slot(OPP_ACTIVE), hp: Num::Lit(20), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
