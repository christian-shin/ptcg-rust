//! Fennel (SV11B): heal 40 damage from each of your Pokémon.
//!
//! Twinleaf: moves to the supporter pile with preventDefault, then one
//! HealEffect per Pokémon (Active first, then the Bench in order). Fixed in
//! phase 4b (R4, Rulings Compendium 851): throws CANNOT_PLAY_THIS_CARD (before
//! the move) when no Pokémon has damage.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Fennel",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::Damaged)],
        steps: &[
            Step::new(Op::ForEach(ForEachSpec { over: SlotSel::Pokemon(Who::Me), body: &[Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::Lit(40), via: HealVia::Effect, clear_conditions: false }))] })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
