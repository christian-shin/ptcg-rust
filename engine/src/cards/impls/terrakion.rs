//! Terrakion (SV11W 54): Retaliate — 50+; 80 more if any of your Pokémon
//! were Knocked Out by damage from an attack during your opponent's last
//! turn. Land Crush — 100.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Terrakion", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    if g.st.players[p].pokemon_knocked_out_by_attack_during_opponents_last_turn {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 80;
        }
    }
    Ok(())
}
