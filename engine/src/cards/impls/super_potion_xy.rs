//! Super Potion (XY, as Super Potion JTG): heal 60 damage from 1 of your
//! Pokémon. If you do, discard an Energy attached to that Pokémon.
//!
//! Twinleaf: only undamaged Pokémon are blocked (phase 4b: Pokémon with no
//! Energy card in `cards` used to be blocked too); a chosen Pokémon with no
//! Energy is just healed; neither prompt can be cancelled since phase 4b (a cancel
//! was choosing nothing; rulings 1778/1853); the Energy choice is on the whole slot list (superType ENERGY);
//! MOVE_CARDS to the discard, then HealEffect 60 (x-and-y file; same flow as
//! the BS port).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SuperPotion@JTG",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" })),
            // A chosen Pokémon without Energy is just healed.
            Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT })),
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(60), via: HealVia::Effect, clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
