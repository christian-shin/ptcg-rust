//! Tapu Koko ex (JTG): Thunder Connect — 60+, 20 more damage for each of
//! your Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TapuKokoex@JTG", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let n = pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += n * 20;
        }
    }
    Ok(())
}
