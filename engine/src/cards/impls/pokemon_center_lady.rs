//! Pokémon Center Lady (FLF): heal 60 damage and remove all Special
//! Conditions from 1 of your Pokémon.
//!
//! Twinleaf: no supporter-turn check in the card; the conditions are wiped
//! directly (`specialConditions = []`) after the HealEffect. Fixed in phase 4b
//! (R4, Rulings Compendium 851): throws CANNOT_PLAY_THIS_CARD when no Pokémon
//! has damage or a Special Condition (a card can't be played for no effect).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokemonCenterLady",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::OneOf(&[SlotPred::Damaged, SlotPred::HasCondition])), msg: "CHOOSE_POKEMON_TO_HEAL" }), hp: Num::Lit(60), via: HealVia::Effect, clear_conditions: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
