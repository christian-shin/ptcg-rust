//! Salamence ex (JTG): Wide Blast — 50 damage to each of the opponent's
//! Benched Pokémon (no Weakness/Resistance). Dragon Impact — 300; discard 2
//! Energy from this Pokémon (ChooseEnergyPrompt for [C][C], no cancel).
//!
//! Twinleaf: Dragon Impact returns early when the Active has no Energy cards
//! attached; otherwise the prompt/discard is the same as
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Salamenceex",
    attacks: &[
        AttackSpec {
            index: 0,
            // Wide Blast: 50 damage to each of the opponent's Benched Pokémon (no Weakness or Resistance).
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Bench(Who::Opp), what: EachWhat::Damage(DamageCalc::Put), amount: Num::Lit(50), ..EachSlotSpec::DEFAULT }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Cost { n: Num::Lit(2), ty: ct::COLORLESS }, when: Cond::Slot(MY_ACTIVE, SlotPred::HasEnergy), ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
