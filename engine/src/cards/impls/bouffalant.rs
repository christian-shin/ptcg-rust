//! Bouffalant (SCR): Curly Wall — if you have any other Bouffalant in play,
//! each of your Basic [C] Pokémon takes 60 less damage from your opponent's
//! attacks (after Weakness and Resistance); only 1 Curly Wall applies.
//! Boundless Power — 130; during your next turn this Pokémon can't attack
//! (THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN: `cannotAttackNextTurnPending` on
//! the Active).
//!
//! Twinleaf quirks kept: every copy reacts from any zone (its owner is the
//! owner of the list holding it); it needs 2+ Pokémon named Bouffalant in
//! that owner's play, then an ability-lock probe; any PutDamageEffect from
//! the opponent's attack on the owner's Basic [C] Pokémon is reduced, once
//! per effect via the `nonstackingDamageReducers` source 'Curly Wall'.
//!
//! Fixed (phase 4b, W4): Boundless Power's lock was missing, and Curly Wall
//! also reduced the owner's own attacks (self-damage).
use crate::cards::prelude::*;
use crate::game::fx_flag;

pub static IMPL: CardImpl = CardImpl { class: "Bouffalant", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Boundless Power
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    let b = match *g.e(e) {
        Effect::PutDamage { b, .. } => b,
        _ => return Ok(()),
    };
    if ignores_defender_effects(g, &b) {
        return Ok(());
    }
    let owner = match g.st.locate(me).and_then(|l| l.owner()) {
        Some(o) => o,
        None => return Ok(()),
    };
    let count = for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).name == "Bouffalant").count();
    if count < 2 {
        return Ok(());
    }
    // Only attacks from the opponent's Pokémon
    if b.player as usize == owner {
        return Ok(());
    }
    if is_ability_blocked(g, owner, me, None) {
        return Ok(());
    }
    let t = b.target;
    if let Some(tc) = g.st.slot_pokemon(t.p as usize, t.s) {
        let d = g.st.cdef(tc);
        if d.card_type.contains(&ct::COLORLESS) && d.stage == Stage::Basic as u8 && t.p as usize == owner {
            if g.fx_flags(e) & fx_flag::CURLY_WALL != 0 {
                return Ok(());
            }
            if let Effect::PutDamage { damage, .. } = g.e_mut(e) {
                *damage = *damage - 60;
            }
            g.set_fx_flag(e, fx_flag::CURLY_WALL);
        }
    }
    Ok(())
}
