//! Archaludon (M2 / PFL 75): Coated Attack — 120; during your opponent's
//! next turn, prevent all damage done to this Pokémon by attacks from Basic
//! Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE with `{ sourceStage: BASIC }`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Archaludon", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let filter = PreventFilter { source_stage: Some(Stage::Basic as u8), source_card_types: None, source_has_ability: false };
        prevent_damage_filtered(g, e, filter)?;
    }
    Ok(())
}
