//! Pokémon Center Lady (MEG 123): heal 60 damage and remove all Special Conditions from 1 of your Pokémon.
//!
//! One Heal of 60 with `clear_conditions` on a Pokémon that has damage or a Special Condition; it can't be played when no
//! Pokémon has either (a Trainer that can't have any effect can't be played, id925).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokemonCenterLady",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::OneOf(&[SlotPred::Damaged, SlotPred::HasCondition])), }), hp: Num::Lit(60), clear_conditions: true })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
