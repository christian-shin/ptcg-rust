//! Salamence ex (JTG 114): Wide Blast — 50 damage to each of your opponent's
//! Benched Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//! Dragon Impact — 300; discard 2 Energy from this Pokémon.
//!
//! Rule: Wide Blast is one Damage event of 50 per Benched Pokémon (cause: this
//! attack; no Weakness or Resistance, APR B-08; each protected Pokémon refuses its
//! own at step 6). Dragon Impact discards 2 Energy of this Pokémon after the
//! damage (you choose which; nothing happens with no Energy attached).
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
