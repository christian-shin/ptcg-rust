//! Mega Lopunny ex (PFL / M2): Gale Thrust — 60+, 170 more if this Pokémon
//! moved from your Bench to the Active Spot this turn. Spiky Hopper — 160,
//! not affected by effects on your opponent's Active Pokémon.
//!
//! Twinleaf: Gale Thrust checks `player.movedToActiveThisTurn` for this
//! card's id. Spiky Hopper reduces its own ApplyWeaknessEffect (Weakness and
//! Resistance apply), zeroes the attack damage, adds the damage straight to
//! the opponent's Active and reduces an AfterDamageEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaLopunnyex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            if g.st.players[p as usize].moved_to_active_this_turn.contains(&me) {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 170;
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        shred(g, e, 160)?;
    }
    Ok(())
}

/// The shared Twinleaf "damage isn't affected by effects on the Defending
/// Pokémon" pattern: own ApplyWeaknessEffect, `effect.damage = 0`, damage
/// added directly to the opponent's Active, then an AfterDamageEffect.
pub fn shred(g: &mut Game, e: EffId, base: i32) -> R {
    let (p, opp, attack, source) = match *g.e(e) {
        Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
        _ => return Ok(()),
    };
    let o = 1 - p as usize;
    let target = SlotRef::new(o, g.st.players[o].active);
    let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
    let (w, _) = g.run_fx(Effect::ApplyWeakness { b, damage: base, ignore_weakness: false, ignore_resistance: false })?;
    let damage = match w {
        Effect::ApplyWeakness { damage, .. } => damage,
        _ => base,
    };
    if let Effect::Attack { damage, .. } = g.e_mut(e) {
        *damage = 0;
    }
    if damage > 0 {
        let a = g.st.players[o].active;
        g.st.players[o].slots[a as usize].damage += damage;
        let target = SlotRef::new(o, a);
        let b = AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::AfterDamage { b, damage })?;
    }
    Ok(())
}
