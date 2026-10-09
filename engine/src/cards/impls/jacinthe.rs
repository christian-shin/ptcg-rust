//! Jacinthe (POR, supporter): heal 150 damage from 1 of your [P] Pokémon.
//!
//! Twinleaf: the supporter and "[P] Pokémon with damage" checks use a
//! CheckPokemonTypeEffect per Pokémon (no `canPlay` quirks matter). The card
//! moves to the supporter pile and the trainer effect is prevented before the
//! (uncancellable) prompt, which blocks every Pokémon that isn't [P] (fixed
//! in R1-10: any of your Pokémon could be chosen); the heal still checks
//! that the chosen one is [P] at resolution.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Jacinthe",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::TypeIs(ct::PSYCHIC), SlotPred::Damaged]))],
        steps: &[
            Step::new(Op::Heal(HealSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(ct::PSYCHIC)), msg: "CHOOSE_POKEMON_TO_HEAL" }), hp: Num::Lit(150), clear_conditions: false })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
