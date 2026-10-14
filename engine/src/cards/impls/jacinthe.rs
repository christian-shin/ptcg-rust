//! Jacinthe (POR, Supporter): heal 150 damage from 1 of your [P] Pokémon.
//!
//! Playable only with a damaged [P] Pokémon (the type as the game checks it, `SlotPred::TypeIs`); the prompt offers only [P]
//! Pokémon, and the heal is one Heal event on the chosen one (cause: this Supporter).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Jacinthe",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::TypeIs(ct::PSYCHIC), SlotPred::Damaged]))],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(ct::PSYCHIC)), purpose: PromptPurpose::HealTarget }), hp: Num::Lit(150), clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
