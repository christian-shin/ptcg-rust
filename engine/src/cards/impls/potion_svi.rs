//! Potion (POR 83, from SVI): heal 30 damage from 1 of your Pokémon.
//!
//! One Heal of 30 on a damaged Pokémon of yours; it can't be played with no damaged Pokémon (id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Potion@POR",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), }), hp: Num::Lit(30), clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
