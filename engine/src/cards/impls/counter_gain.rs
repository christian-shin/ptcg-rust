//! Counter Gain (LOT / ASC, tool): if you have more Prize cards remaining
//! than your opponent, the attacks of the Pokémon this card is attached to
//! cost [C] less.
//!
//! Twinleaf: the tool must be on the attacker's Active; IS_TOOL_BLOCKED must
//! not hold (phase 4b: it used to be a bare ToolEffect stub that ignored the
//! "Stadiums and Tools have no effect" turns); then the first [C] of the cost
//! is removed (the card says [C] less) when the attacker has more Prize cards
//! left.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CounterGain", mask: mask(&[k::CHECK_ATTACK_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::CheckAttackCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    if g.st.players[p].prize_left() > g.st.players[1 - p].prize_left() {
        // Applied once, with the other cost changes, after all handlers ran (D-11, D-12).
        if let Effect::CheckAttackCost { reduction, .. } = g.e_mut(e) {
            *reduction += 1;
        }
    }
    Ok(())
}
