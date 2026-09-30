//! Air Balloon (SSH, tool): the Retreat Cost of the Pokémon this card is
//! attached to is [C][C] less.
//!
//! Twinleaf quirk kept: `cost.splice(indexOf(C), 2)` removes the first
//! Colorless and whatever single entry follows it.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "AirBalloon", mask: mask(&[k::CHECK_RETREAT_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, cost) = match *g.e(e) {
        Effect::CheckRetreatCost { p, cost } => (p as usize, cost),
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    let index = cost.iter().position(|c| *c == ct::COLORLESS);
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    if let Some(i) = index {
        let mut out: crate::effects::Cost = SVec::new();
        for (j, c) in cost.iter().enumerate() {
            if j < i || j >= i + 2 {
                out.push(*c);
            }
        }
        if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
            *cost = out;
        }
    }
    Ok(())
}
