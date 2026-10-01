//! Dipplin (SCR 13): Coated Attack — 20; during your opponent's next turn,
//! prevent all damage done to this Pokémon by attacks from Basic Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE with `{ sourceStage: BASIC }`. Two `Dipplin`
//! classes exist; this port is bound to SCR.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dipplin@SCR", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        prevent_damage_filtered(g, e, PreventFilter { source_stage: Some(Stage::Basic as u8), source_card_types: None })?;
    }
    Ok(())
}
