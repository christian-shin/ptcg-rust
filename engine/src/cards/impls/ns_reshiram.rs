//! N's Reshiram (JTG / ASC): Powerful Rage - 20 damage for each damage
//! counter on this Pokémon. Virtuous Flame - 170.
//!
//! Twinleaf: `effect.damage = player.active.damage * 2` (the Active of the
//! attacking player).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsReshiram", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let d = g.st.players[p].slots[a as usize].damage * 2;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = d;
        }
    }
    Ok(())
}
