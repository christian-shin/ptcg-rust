//! Super Potion (BS): discard 1 Energy card attached to 1 of your own Pokémon in order to remove up to 4 damage counters from
//! that Pokémon.
//!
//! The discard is the cost, `Step::cost` (a LeavePlay of the Energy, APR E-30): only a damaged Pokémon with an Energy can be
//! chosen, the choices can be declined (then nothing happens), and the heal is the card's effect.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SuperPotion@BS",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            // Both choices can be cancelled (nothing happens).
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Cancelable(&SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Damaged, SlotPred::HasEnergy]))),
                purpose: PromptPurpose::HealTarget,
            })),
            Step::cost(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: true, energies_only: false }, into: Some(0), ..DiscardEnergySpec::DEFAULT })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(40), clear_conditions: false }))],
                no: &[],
            })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
