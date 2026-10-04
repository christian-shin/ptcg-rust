//! Ho-Oh (SSP): Flap — 50. Shining Blaze — 100+; 100 more if you have any Tera
//! Pokémon on your Bench.
//!
//! Fixed (phase 4b, W4): Twinleaf counted the Active Pokémon too.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HoOh", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let tera = g.st.players[p].bench.iter().any(|s| g.st.slot_pokemon(p, *s).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_TERA)).unwrap_or(false));
        if tera {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 100;
            }
        }
    }
    Ok(())
}
