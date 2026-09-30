//! Stunfisk ex (ASC): Big Bite — 30; during your opponent's next turn the
//! Defending Pokémon can't retreat. Flopping Trap — 100+; 100 more if this
//! Pokémon has any damage counters on it (Twinleaf reads `player.active`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Stunfiskex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return block_retreat(g, e);
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            if g.st.slot(p, a).damage > 0 {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 100;
                }
            }
        }
    }
    Ok(())
}
