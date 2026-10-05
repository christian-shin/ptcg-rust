//! Gravity Gemstone (SCR, tool): as long as the Pokémon this card is
//! attached to is in the Active Spot, the Retreat Cost of both Active
//! Pokémon is [C] more.
//!
//! Twinleaf: on a CheckRetreatCostEffect, unless the tool is blocked for the
//! effect's player, a [C] is pushed when either Active holds this tool, except
//! when the Active's printed Retreat Cost is not empty but the cost is already
//! empty (an effect such as Skyliner or Metal Bridge set it to none; phase 4b,
//! ruling n=1617: it can't be increased).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "GravityGemstone", mask: mask(&[k::CHECK_RETREAT_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let o = 1 - p;
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    let pa = g.st.players[p].active;
    let oa = g.st.players[o].active;
    if g.st.slot(p, pa).tools.contains(me) || g.st.slot(o, oa).tools.contains(me) {
        let printed = g.st.slot_pokemon(p, pa).map(|c| g.st.cdef(c).retreat.len()).unwrap_or(0);
        if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
            if printed > 0 && cost.is_empty() {
                return Ok(());
            }
            cost.push(ct::COLORLESS);
        }
    }
    Ok(())
}
