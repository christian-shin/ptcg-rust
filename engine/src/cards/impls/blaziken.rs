//! Blaziken (DRI 42): Heat Blast - 70. Inferno Kick Flurry - 120; discard 2 Energy from this Pokémon, and 120 damage
//! to 1 of your opponent's Benched Pokémon.
//!
//! The Bench damage is part of the attack: one Damage event on the chosen Benched Pokémon (no Weakness or Resistance on
//! the Bench, APR B-08; the target is chosen before the Energy). The 2 Energy are then discarded from this Pokémon: a
//! LeavePlay per card (D1), the player chooses which, cause: this attack.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Blaziken",
    // Inferno Kick Flurry: discard 2 Energy from this Pokémon, and 120 damage to 1 of your opponent's
    // Benched Pokémon (the damage target is chosen before the Energy to discard).
    attacks: &[AttackSpec {
        index: 1,
        steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), }),
                hp: Num::Lit(120),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
