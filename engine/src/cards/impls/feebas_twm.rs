//! Feebas (TWM): Flail — 10x; `effect.damage = player.active.damage` (the
//! damage on the Active in HP, i.e. 10 per damage counter).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Feebas@Feebas TWM", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let d = g.st.slot(p, a).damage;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = d;
        }
    }
    Ok(())
}
