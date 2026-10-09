//! Shuckle (MEG): Fermented Juice — once during your turn, if this Pokémon
//! has any [G] Energy attached, heal 30 damage from 1 of your Pokémon.
//! Rollout — 30.
//!
//! Twinleaf: throws CANNOT_USE_POWER without [G] provided or without a
//! damaged Pokémon, then USE_ABILITY_ONCE_PER_TURN (POWER_ALREADY_USED) and
//! ABILITY_USED before a non-cancellable ChoosePokemonPrompt (undamaged
//! Pokémon blocked) and a HealEffect of 30.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shuckle@MEG",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("FERMENTED_JUICE_MARKER"),
        needs: &[Cond::Slot(SlotExpr::This, SlotPred::Provides(ct::GRASS)), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Damaged)],
        steps: &[Step::new(Op::Heal(HealSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" }),
            hp: Num::Lit(30),
            clear_conditions: false,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
