//! Duraludon (PFL / M2 74): Hyper Beam — 70; discard an Energy from your
//! opponent's Active Pokémon.
//!
//! Twinleaf: DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON (no prompt when
//! the Active has no Energy card).
use crate::cards::prelude::*;
use super::trubbish::{discard_an_energy_from_opponents_active, discard_chosen};

pub static IMPL: CardImpl = CardImpl { class: "Duraludon@PFL", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        g.retain_fx(e);
        return discard_an_energy_from_opponents_active(g, me, e, 1);
    }
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage == 1 {
        return discard_chosen(g, f, results);
    }
    Ok(())
}
