//! Shuckle (MEG 11): Fermented Juice - once during your turn, if this Pokémon has any [G] Energy attached, you may use this
//! Ability. Heal 30 damage from 1 of your Pokémon. Rollout - 30.
//!
//! An Ability use (once per turn) that needs this Pokémon to provide [G] (the Energy provided: a Prism Energy on a Basic Pokémon
//! counts) and a damaged Pokémon of yours; one Heal of 30 on the Pokémon the player chooses.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shuckle@MEG",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("FERMENTED_JUICE_MARKER"),
        needs: &[Cond::Slot(SlotExpr::This, SlotPred::Provides(ct::GRASS))],
        steps: &[Step::new(Op::Heal(HealSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" }),
            hp: Num::Lit(30),
            clear_conditions: false,
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
