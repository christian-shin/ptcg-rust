//! Dartrix (SFA): United Wings — 20× the number of Pokémon in your discard
//! pile with the United Wings attack. Cutting Wind — 30.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dartrix@SFA", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let n = g.st.players[p]
        .discard
        .iter()
        .filter(|c| {
            let d = g.st.cdef(*c);
            d.is_pokemon() && d.attacks.iter().any(|a| a.name == "United Wings")
        })
        .count() as i32;
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = n * 20;
    }
    Ok(())
}
