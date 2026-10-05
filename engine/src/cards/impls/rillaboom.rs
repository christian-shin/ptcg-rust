//! Rillaboom (TWM 16): Drum Beating — 60; during your opponent's next turn,
//! attacks used by the Defending Pokémon cost [C] more, and its Retreat Cost
//! is [C] more. Wood Hammer — 180; this Pokémon also does 50 damage to itself.
//!
//! Twinleaf: DEFENDING_POKEMON_ATTACKS_COST_MORE / RETREAT_COSTS_MORE write
//! `...Pending = 1` on the opponent's Active before reducing their
//! EffectOfAttackEffects (so the pending value survives a prevented effect,
//! without an attacker id, and is never armed).
//! Phase 4b (R6): the counters are armed when the attacker ends its turn and
//! expire when the defender ends its turn (`phase.rs`), and the core
//! (`check.rs`: CheckAttackCost / CheckRetreatCost) pushes one [C] per point.
//! Before, they were armed at the defender's end of turn (live only in the
//! attacker's next turn, so Drum Beating never bit) and only read by the
//! crossed handlers of every Rillaboom card anywhere (one extra [C] per card).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Rillaboom",
    mask: mask(&[k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

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
    Ok(())
}
