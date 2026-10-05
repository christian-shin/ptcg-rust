//! Munkidori ex (SFA): Oh No You Don't — if this Pokémon is Knocked Out and
//! you have Pecharunt ex in play, the opponent takes 1 fewer Prize card.
//! Dirty Headbutt — 190; during your next turn this Pokémon can't use it.
//!
//! Twinleaf (phase 4b): a KnockOutEffect on this card's slot counts only
//! during the opponent's ATTACK phase with the owner carrying
//! DAMAGE_DEALT_MARKER (Knocked Out by damage from an attack; it used to count
//! any KO, e.g. Poison), and "Pecharunt ex in play" is a scan of the owner's
//! Pokémon (it used the owner's `pecharuntexIsInPlay` flag, which Pecharunt ex
//! set only while its owner's Active had a Special Condition and never
//! cleared).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Munkidoriex", mask: mask(&[k::KNOCK_OUT, k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::KnockOut { p, target, .. } = *g.e(e) {
        if g.st.slot(target.p as usize, target.s).cards.contains(me) {
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            if g.st.phase != GamePhase::Attack
                || g.st.active_player as usize != 1 - p
                || !g.st.players[p].marker.has(crate::markers::DAMAGE_DEALT_MARKER)
            {
                return Ok(());
            }
            let has_pecharunt = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).name == "Pecharunt ex");
            if has_pecharunt {
                if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
                    *prize_count -= 1;
                }
            }
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !pending.iter().any(|n| *n == "Dirty Headbutt") {
                pending.push("Dirty Headbutt");
            }
        }
    }
    Ok(())
}
