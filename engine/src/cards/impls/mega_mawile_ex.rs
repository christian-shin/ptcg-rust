//! Mega Mawile ex (M1L / MEG 94): Gobble Down — 80x for each Prize card you
//! have taken. Huge Bite — 260; if the opponent's Active already has damage
//! counters, the base damage is 30.
//!
//! Twinleaf: both attacks overwrite `effect.damage`.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaMawileEx", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let taken = 6 - g.st.players[p].prize_left() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = taken * 80;
        }
    }
    if was_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[o].active;
        if g.st.slot(o, a).damage > 0 {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage = 30;
            }
        }
    }
    Ok(())
}
