//! Riolu (M1L / MEG 76): Accelerating Stab — 30. During your next turn, this
//! Pokémon can't use Accelerating Stab.
//!
//! Twinleaf has several `Riolu` classes; this port is bound to M1L.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Riolu@Riolu M1L", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            push_pending(g, p as usize, "Accelerating Stab");
        }
    }
    Ok(())
}

/// `if (!player.active.cannotUseAttacksNextTurnPending.includes(name)) push(name)`.
pub fn push_pending(g: &mut Game, p: usize, name: &'static str) {
    let a = g.st.players[p].active;
    let v = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
    if !v.contains(&name) {
        v.push(name);
    }
}
