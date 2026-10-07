//! Gravity Gemstone (SCR, tool): as long as the Pokémon this card is
//! attached to is in the Active Spot, the Retreat Cost of both Active
//! Pokémon is [C] more.
//!
//! Twinleaf: on a CheckRetreatCostEffect, unless the tool is blocked for the
//! effect's player, a [C] is pushed when either Active holds this tool. A cost that
//! an effect set to none (Skyliner, Metal Bridge; ruling 1617) is emptied by the core
//! afterwards (`no_cost`); a cost reduced to 0 by "less" effects is calculated
//! together with the increase (Advanced Rulebook D-11, D-12; ruling 836).
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
        if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
            cost.push(ct::COLORLESS);
        }
    }
    Ok(())
}
