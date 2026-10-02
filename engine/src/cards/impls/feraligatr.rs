//! Feraligatr (TEF): Torrential Heart — once during your turn, you may put 5
//! damage counters on this Pokémon; attacks used by this Pokémon do 120 more
//! damage this turn. Giant Wave — 160; this Pokémon can't use Giant Wave
//! during your next turn.
//!
//! Twinleaf: the +120 applies to every AttackEffect whose source list holds
//! this card while the player marker is set (not only Giant Wave). The Ability
//! throws BLOCKED_BY_EFFECT when the marker is already set (it is not a
//! USE_ABILITY_ONCE_PER_TURN call) and adds 50 damage to the slot holding
//! this card without any check for Knock Out. Giant Wave pushes its name onto
//! the player's Active `cannotUseAttacksNextTurnPending` if missing.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Feraligatr", mask: mask(&[k::END_TURN, k::ATTACK, k::POWER]), reduce, resume: None, coin: None, can_play: None };

fn heart() -> crate::markers::MarkerName {
    crate::marker!("TORRENTIAL_HEART_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    remove_marker_at_end_of_turn(g, e, heart(), me);

    if let Effect::Attack { p, source, .. } = *g.e(e) {
        if g.st.slot(source.p as usize, source.s).cards.contains(me) && g.st.players[p as usize].marker.has_from(heart(), me) {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 120;
            }
        }
    }

    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            let pending = &mut g.st.players[p].slots[a as usize].cannot_use_attacks_next_turn_pending;
            if !pending.iter().any(|n| *n == "Giant Wave") {
                pending.push("Giant Wave");
            }
        }
    }

    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if g.st.players[p].marker.has_from(heart(), me) {
            bail!("BLOCKED_BY_EFFECT");
        }
        let slot = match g.st.players[p].in_play().iter().copied().find(|s| g.st.slot(p, *s).cards.contains(me)) {
            Some(s) => s,
            None => return Ok(()),
        };
        g.st.players[p].slots[slot as usize].damage += 50;
        g.st.players[p].marker.add(heart(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
        ability_used(g, p, me);
    }
    Ok(())
}
