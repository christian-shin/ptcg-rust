//! Latias ex (SSP): Skyliner - your Basic Pokémon in play have no Retreat
//! Cost. Eon Blade - 200; during your next turn this Pokémon can't attack.
//!
//! Twinleaf checks only whether the retreating (Active) Pokémon is Basic,
//! and only while this Latias ex is the top Pokémon of one of the owner's
//! in-play slots.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Latiasex", mask: mask(&[k::CHECK_RETREAT_COST, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckRetreatCost { p, .. } = *g.e(e) {
        let p = p as usize;
        let owner = g.st.owner(me);
        let active = match g.st.active_pokemon(p) {
            Some(c) => c,
            None => return Ok(()),
        };
        if owner != p {
            return Ok(());
        }
        let in_play = g.st.players[owner].in_play().iter().any(|s| g.st.slot_pokemon(owner, *s) == Some(me));
        if !in_play {
            return Ok(());
        }
        if !is_ability_blocked(g, p, me, None) && g.st.cdef(active).stage == Stage::Basic as u8 {
            if let Effect::CheckRetreatCost { cost, .. } = g.e_mut(e) {
                cost.clear();
            }
        }
        return Ok(());
    }
    // THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN(player).
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
