//! Super Potion (JTG 158): Item. Heal 60 damage from 1 of your Pokémon. If you do, discard an Energy attached to that
//! Pokémon.
//!
//! The heal is a RemoveCounters event (cause: this Item) and "if you do" is `Cond::Done` (APR E-20): a refused heal
//! (Yveltal's Life-Locked on the Active Pokémon) discards nothing. The Energy is a LeavePlay (D1) to its owner's discard
//! pile; a chosen Pokémon without Energy is just healed. Neither choice can be cancelled (an Item's choice is at least 1,
//! id2301).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SuperPotion@JTG",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged) }, purpose: PromptPurpose::HealTarget })),
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(60), clear_conditions: false })),
            // "If you do": the heal happened (a Pokémon whose healing is prevented keeps its Energy). A chosen Pokémon without
            // Energy is just healed.
            Step::new(Op::If(IfSpec {
                cond: Cond::Done,
                yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
