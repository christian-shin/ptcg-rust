//! Zeraora (DRI): Scratch — 20. Thunder Raid — discard all Energy from this
//! Pokémon; 210 damage to 1 of the opponent's Benched Pokémon ex.
//!
//! Only the opponent's Benched Pokémon count as "Benched ex" (an ex Active Pokémon alone doesn't make the target
//! reachable). With no Benched ex the attack is still usable (ruling 1790): the Energy is discarded and there is no damage
//! and no prompt. Otherwise one Damage event on the chosen Benched ex (no Weakness or Resistance), with the attack as its
//! cause.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Zeraora@DRI",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Top(Pred::Tag(tag::POKEMON_EX_LOWER))), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(210), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
