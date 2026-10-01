//! Incineroar ex (TEF): Hustle Play — attacks used by this Pokémon cost [C]
//! less for each of your opponent's Benched Pokémon. Blaze Blast — 240; the
//! opponent's Active Pokémon is now Burned.
//!
//! Twinleaf: the CheckAttackCostEffect handler has no attack check (any
//! attack cost check while this is the Active) and does
//! `cost.splice(cost.indexOf(C), benched)`; with no [C] in the cost,
//! `indexOf` is -1 and `splice(-1, n)` removes the last element.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Incineroarex", mask: mask(&[k::CHECK_ATTACK_COST, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, .. } = *g.e(e) {
        let p = p as usize;
        let a = g.st.players[p].active;
        if g.st.slot_pokemon(p, a) != Some(me) {
            return Ok(());
        }
        if is_ability_blocked(g, p, me, Some(0)) {
            return Ok(());
        }
        let o = 1 - p;
        let benched = g.st.players[o].bench.iter().filter(|s| !g.st.players[o].slots[**s as usize].cards.is_empty()).count();
        if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
            let len = cost.len() as i64;
            let idx = cost.iter().position(|t| *t == ct::COLORLESS).map(|i| i as i64).unwrap_or(-1);
            let start = if idx < 0 { (len + idx).max(0) } else { idx.min(len) };
            let end = (start + benched as i64).min(len);
            let mut out = SVec::new();
            for (i, t) in cost.iter().enumerate() {
                let i = i as i64;
                if i < start || i >= end {
                    out.push(*t);
                }
            }
            *cost = out;
        }
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Burned])?;
    }
    Ok(())
}
