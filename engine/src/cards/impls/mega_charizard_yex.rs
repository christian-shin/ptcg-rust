//! Mega Charizard Y ex (ASC 22): Explosion Y - discard 3 Energy from this Pokémon, and 280 damage to 1 of your opponent's
//! Pokémon (the Bench takes no Weakness/Resistance).
//!
//! The damage to the chosen Pokémon is one Damage event (cause: this attack), then the 3 Energy are discarded after it:
//! a LeavePlay per card (D1), the player chooses which.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaCharizardYEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::PokemonBenchFirst(Who::Opp), }),
                hp: Num::Lit(280),
                target_damage_mul: 0,
                calc: DamageCalc::Auto,
                when: Cond::True,
            })),
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 3, ty: crate::types::ct::COLORLESS }, ..DiscardEnergySpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
