//! Air Balloon (SSH, tool): the Retreat Cost of the Pokémon this card is
//! attached to is [C][C] less.
//!
//! The reduction is added to `CheckRetreatCostEffect.costReduction` and applied by the core
//! after every handler ran, together with the increases (Advanced Rulebook D-11, D-12).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "AirBalloon", mask: mask(&[k::CHECK_RETREAT_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    // Calculated together with the effects that make the cost more (Advanced Rulebook D-11, D-12).
    if let Effect::CheckRetreatCost { reduction, .. } = g.e_mut(e) {
        *reduction += 2;
    }
    Ok(())
}
