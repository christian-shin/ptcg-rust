//! Rillaboom (TWM 16): Drum Beating — 60; during your opponent's next turn,
//! attacks used by the Defending Pokémon cost [C] more, and its Retreat Cost
//! is [C] more. Wood Hammer — 180; this Pokémon also does 50 damage to itself.
//!
//! Twinleaf quirks kept: DEFENDING_POKEMON_ATTACKS_COST_MORE /
//! RETREAT_COSTS_MORE write `...Pending = 1` on the opponent's Active before
//! reducing their EffectOfAttackEffects (so the pending value survives a
//! prevented effect, without an attacker id). No core rule reads the
//! counters: every Rillaboom card anywhere reacts to a CheckRetreatCost /
//! CheckAttackCost of a player whose Active has the *attack* / *retreat*
//! counter (crossed) set, inserting a [C] at the first [C] (or appending).
//! The EndTurnEffect arming (`phase.rs`) makes the counters live during the
//! *attacker's* following turn, so they only bite the defender after a
//! second Drum Beating (pending re-set, the clear is skipped), or forever
//! when the effect was prevented (no attacker id to clear it).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Rillaboom",
    mask: mask(&[k::ATTACK, k::CHECK_RETREAT_COST, k::CHECK_ATTACK_COST]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn add_colorless(cost: &mut crate::effects::Cost) {
    match cost.position(&ct::COLORLESS) {
        Some(i) => cost.insert(i, ct::COLORLESS),
        None => cost.push(ct::COLORLESS),
    }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let o = opp as usize;
        let a = g.st.players[o].active;
        g.st.players[o].slots[a as usize].attack_cost_increase_next_turn_pending = 1;
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, a) };
        g.run_fx(Effect::IncreaseAttackCostNextTurn { b })?;
        let a = g.st.players[o].active;
        g.st.players[o].slots[a as usize].retreat_cost_increase_next_turn_pending = 1;
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(o, a) };
        g.run_fx(Effect::IncreaseRetreatCostNextTurn { b })?;
        return Ok(());
    }
    if was_attack_used(g, e, 1, me) {
        let (p, opp, attack, source) = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let a = g.st.players[p as usize].active;
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: SlotRef::new(p as usize, a) };
        g.run_fx(Effect::DealDamage { b, damage: 50 })?;
        return Ok(());
    }
    match *g.e(e) {
        Effect::CheckRetreatCost { p, .. } => {
            let p = p as usize;
            let a = g.st.players[p].active;
            if g.st.slot(p, a).attack_cost_increase_next_turn > 0 && g.st.slot_pokemon(p, a).is_some() {
                if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
                    add_colorless(cost);
                }
            }
        }
        Effect::CheckAttackCost { p, .. } => {
            let p = p as usize;
            let a = g.st.players[p].active;
            if g.st.slot(p, a).retreat_cost_increase_next_turn > 0 && g.st.slot_pokemon(p, a).is_some() {
                if let Effect::CheckAttackCost { cost, .. } = g.e_mut(e) {
                    add_colorless(cost);
                }
            }
        }
        _ => {}
    }
    Ok(())
}
