//! Hop's Choice Band (JTG, tool): attacks used by the Hop's Pokémon this card
//! is attached to cost [C] less and do 30 more damage to your opponent's
//! Active Pokémon (before applying Weakness and Resistance).
//!
//! Twinleaf: on CheckAttackCostEffect (holder is the player's Active) the
//! tool probe runs first, then the first [C] is removed if the Active is a
//! Hop's Pokémon (the handler always returns, so a cost check never reaches
//! the damage branch). On DealDamageEffect from the holder's slot, after the
//! probe, damage to the opponent's Active gets +30 for a Hop's holder when the
//! damage is above 0 (phase 4b: the guard was missing).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HopsChoiceBand", mask: mask(&[k::CHECK_ATTACK_COST, k::DEAL_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckAttackCost { p, .. } => {
            let p = p as usize;
            let a = g.st.players[p].active;
            if !g.st.slot(p, a).tools.contains(me) {
                return Ok(());
            }
            if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
                return Ok(());
            }
            let hops = g.st.slot_pokemon(p, a).map(|c| g.st.cdef(c).has_tag(tag::HOPS)).unwrap_or(false);
            if hops {
                // Applied once, with the other cost changes, after all handlers ran (D-11, D-12).
                if let Effect::CheckAttackCost { reduction, .. } = g.e_mut(e) {
                    *reduction += 1;
                }
            }
            Ok(())
        }
        Effect::DealDamage { b, damage: dealt } => {
            if !g.st.slot(b.source.p as usize, b.source.s).tools.contains(me) {
                return Ok(());
            }
            let p = b.player as usize;
            let o = 1 - p;
            if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
                return Ok(());
            }
            let t = b.target;
            if !(t.p as usize == o && t.s == g.st.players[o].active) {
                return Ok(());
            }
            let hops = g.st.slot_pokemon(b.source.p as usize, b.source.s).map(|c| g.st.cdef(c).has_tag(tag::HOPS)).unwrap_or(false);
            if hops && dealt > 0 {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 30;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
