//! Miraidon ex (TEF): Repulsion Bolt — 60+; 100 more if the opponent's Active
//! has damage counters. Cyber Drive — 220; THIS_POKEMON_CANNOT_USE_THIS_ATTACK_NEXT_TURN.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Miraidonex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            let o = opp as usize;
            let a = g.st.players[o].active;
            if g.st.slot(o, a).damage > 0 {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 100;
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let v = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !v.iter().any(|n| *n == "Cyber Drive") {
                v.push("Cyber Drive");
            }
        }
    }
    Ok(())
}
