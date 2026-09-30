//! Munkidori ex (SFA): Oh No You Don't — if this Pokémon is Knocked Out and
//! you have Pecharunt ex in play, the opponent takes 1 fewer Prize card.
//! Dirty Headbutt — 190; during your next turn this Pokémon can't use it.
//!
//! Twinleaf: any KnockOutEffect on this card's slot counts (not only attack
//! damage), and "Pecharunt ex in play" is the owner's `pecharuntexIsInPlay`
//! flag (set by Pecharunt ex, never cleared).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Munkidoriex", mask: mask(&[k::KNOCK_OUT, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::KnockOut { p, target, .. } = *g.e(e) {
        if g.st.slot(target.p as usize, target.s).cards.contains(me) {
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            if g.st.players[p].pecharuntex_is_in_play {
                if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
                    *prize_count -= 1;
                }
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !pending.iter().any(|n| *n == "Dirty Headbutt") {
                pending.push("Dirty Headbutt");
            }
        }
    }
    Ok(())
}
