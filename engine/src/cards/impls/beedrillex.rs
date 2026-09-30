//! Beedrill ex (CRI / M4): Rumbling Bees — 110× the number of your Beedrill
//! and Beedrill ex in play.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Beedrillex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let n = for_each_pokemon(g, p, PlayerType::BottomPlayer)
        .iter()
        .filter(|(_, c, _)| {
            let name = g.st.cdef(*c).name;
            name == "Beedrill" || name == "Beedrill ex"
        })
        .count() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = 110 * n;
    }
    Ok(())
}
