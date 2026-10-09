//! Potion (SVI, as Potion POR): heal 30 damage from 1 of your Pokémon.
//!
//! Twinleaf (scarlet-and-violet file): same flow as the BS port (undamaged
//! Pokémon are blocked; no cancel), HealEffect 30.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Potion@POR",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" }), hp: Num::Lit(30), clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
