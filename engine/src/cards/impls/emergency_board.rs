//! Rescue Board (TEF, Pokémon Tool): the Retreat Cost of the Pokémon this
//! card is attached to is [C] less. If that Pokémon's remaining HP is 30 or
//! less, it has no Retreat Cost.
//!
//! Twinleaf: the tool block probe comes after the top-card lookup.
//!
//! Fixed (phase 4b, R2): the remaining HP was the printed HP minus the
//! Active's damage; it is now the current HP (CheckHpEffect: Stadium, Ability
//! and Energy bonuses) minus the damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EmergencyBoard", mask: mask(&[k::CHECK_RETREAT_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    let pokemon = g.st.slot_pokemon(p, a);
    if is_tool_blocked(g, p, me) {
        return Ok(());
    }
    if pokemon.is_some() {
        let remaining = crate::engine::check::check_hp(g, p, a)? - g.st.slot(p, a).damage;
        if remaining <= 30 {
            if let Effect::CheckRetreatCost { cost, no_cost, .. } = g.e_mut(e) {
                *cost = crate::effects::Cost::new();
                *no_cost = true;
            }
        } else if let Effect::CheckRetreatCost { reduction, .. } = g.e_mut(e) {
            *reduction += 1;
        }
    }
    Ok(())
}
