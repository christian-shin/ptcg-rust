//! Crabominable (SCR): Food Prep — attacks used by this Pokémon cost [C] less
//! for each Kofu card in your discard pile. Haymaker — 250; during your next
//! turn, this Pokémon can't use Haymaker.
//!
//! Twinleaf: on CheckAttackCostEffect (any attack of that player's Active),
//! when this is the player's Active and a PowerEffect for Food Prep passes,
//! `cost.splice(cost.indexOf(C), kofuCount)` — with no [C] left the index is
//! -1, so the last entry is removed (at most one).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Crabominable", mask: mask(&[k::CHECK_ATTACK_COST, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, .. } = *g.e(e) {
        let p = p as usize;
        if g.st.active_pokemon(p) != Some(me) {
            return Ok(());
        }
        // `store.reduceEffect(state, new PowerEffect(player, this.powers[0], this))` in a try/catch.
        if g.run_fx(Effect::Power { p: p as u8, power: PowerRef { card: me, index: 0 }, card: me, target: None, probe: false }).is_err() {
            return Ok(());
        }
        let kofu = g.st.players[p]
            .discard
            .iter()
            .filter(|c| {
                let d = g.st.cdef(*c);
                d.is_trainer() && d.name == "Kofu"
            })
            .count();
        // Applied once, with the other cost changes, after all handlers ran (D-11, D-12).
        if let Effect::CheckAttackCost { reduction, .. } = g.e_mut(e) {
            *reduction = reduction.saturating_add(kofu as u8);
        }
        return Ok(());
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !pending.iter().any(|n| *n == "Haymaker") {
                pending.push("Haymaker");
            }
        }
    }
    Ok(())
}
