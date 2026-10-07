//! Sparkling Crystal (SCR, ACE SPEC Pokémon Tool): when the Tera Pokémon this
//! card is attached to uses an attack, that attack costs 1 Energy less.
//!
//! Twinleaf: on a CheckAttackCostEffect while the tool is on the player's
//! Active: a ToolEffect stub (not IS_TOOL_BLOCKED) must not throw, the Active
//! must be Tera; the provided Energy units (CheckProvidedEnergyEffect on the
//! Active) pay each printed cost slot in order ([C] with any unit, a typed
//! slot with a matching unit, else with a rainbow unit); if the covered slots
//! number at least cost length - 1 the cost becomes the covered units.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SparklingCrystal", mask: mask(&[k::CHECK_ATTACK_COST]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match *g.e(e) {
        Effect::CheckAttackCost { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[p].active;
    if !g.st.slot(p, a).tools.contains(me) {
        return Ok(());
    }
    let pokemon = g.st.slot_pokemon(p, a);
    if g.run_fx(Effect::Tool { p: p as u8, card: me }).is_err() {
        return Ok(());
    }
    let c = match pokemon {
        Some(c) if g.st.cdef(c).has_tag(tag::POKEMON_TERA) => c,
        _ => return Ok(()),
    };
    let _ = c;
    // "Costs 1 Energy less" (any type): applied by the core with the other cost changes after all handlers ran
    // (check.rs), whatever the handler order (Advanced Rulebook D-11, D-12).
    if let Effect::CheckAttackCost { any_reduction, .. } = g.e_mut(e) {
        *any_reduction = true;
    }
    Ok(())
}
