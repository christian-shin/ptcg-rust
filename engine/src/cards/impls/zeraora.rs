//! Zeraora (DRI 78): Scratch: 20. Thunder Raid: discard all Energy from this Pokémon. This attack does 210 damage to 1 of your
//! opponent's Benched Pokémon ex.
//!
//! Only the opponent's Benched Pokémon count as "Benched ex" (an ex Active Pokémon alone doesn't make the target reachable).
//! With no Benched ex the attack is still usable (id362): the Energy is discarded (LeavePlay events, this attack their cause) and there
//! is no damage and no prompt. Otherwise one Damage event on the chosen Benched ex (no Weakness or Resistance), this attack its
//! cause.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Zeraora@DRI",
    attacks: &[
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Top(Pred::PokemonEx)), purpose: PromptPurpose::DamageTarget }), hp: Num::Lit(210), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
