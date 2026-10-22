//! Honchkrow (PFL 58): Wind of Darkness - 30. Sniping Feathers - discard 2 Energy from this Pokémon, and this attack does 120 damage to 1 of your opponent's Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! Two Energy of any type are discarded from this Pokémon, then a chosen opponent's Pokémon takes 120 with `DamageCalc::Auto` (Cramorant TWM 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Honchkrow",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }), hp: Num::Lit(120), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
