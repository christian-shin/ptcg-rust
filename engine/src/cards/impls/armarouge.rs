//! Armarouge (SSP): Combustion — 50. Crimson Blaster — discard all [R] Energy
//! from this Pokémon, and 180 damage to 1 of your opponent's Benched Pokémon.
//!
//! The Energy is discarded first, then one Damage event on the chosen Benched Pokémon (the Active Pokémon can't be
//! chosen; no Weakness or Resistance on the Bench), with the attack as its cause.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Armarouge",
    // Crimson Blaster: discard all [R] Energy from this Pokémon, and 180 damage to 1 of your
    // opponent's Benched Pokémon.
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Provides(ct::FIRE), ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                hp: Num::Lit(180),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
