//! Litwick (PBL / M5): Will-O-Wisp - 20. No effects (Twinleaf's
//! `reduceEffect` only returns the state).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Litwick", mask: mask(&[]), reduce, resume: None, coin: None, can_play: None };

fn reduce(_g: &mut Game, _me: CardId, _e: EffId) -> R {
    Ok(())
}
