//! Potion (BS): remove up to 2 damage counters from 1 of your Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Potion@BS",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Heal(HealSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" }),
            hp: Num::Lit(20),
            via: HealVia::Effect,
            clear_conditions: false,
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
