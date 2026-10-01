//! Unown (30C): Mysterious Signal — 40; if your opponent's Pokémon is
//! Knocked Out by damage from this attack, take 1 more Prize card.
//!
//! Twinleaf: IF_OPPONENTS_POKEMON_KO_BY_ATTACK_DAMAGE_TAKE_MORE_PRIZES with
//! `attackName`; every Unown copy (any zone) reacts to the KnockOutEffect:
//! the target must be a Pokémon in the owner's Active/Bench, the phase must
//! be ATTACK with the attacker active, the owner carries DAMAGE_DEALT_MARKER
//! and the attacker's `playerLastAttack` is this card's Mysterious Signal.
use crate::cards::prelude::*;
use crate::markers::DAMAGE_DEALT_MARKER;

pub static IMPL: CardImpl = CardImpl { class: "Unown@30C", mask: mask(&[k::KNOCK_OUT]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (owner, target) = match *g.e(e) {
        Effect::KnockOut { p, target, .. } => (p as usize, target),
        _ => return Ok(()),
    };
    let attacker = 1 - owner;
    let pl = &g.st.players[owner];
    let defending = target.p as usize == owner && (pl.active == target.s || pl.bench.iter().any(|b| *b == target.s));
    if !defending {
        return Ok(());
    }
    if g.st.phase != GamePhase::Attack || g.st.active_player as usize != attacker {
        return Ok(());
    }
    if !pl.marker.has(DAMAGE_DEALT_MARKER) {
        return Ok(());
    }
    match g.st.player_last_attack[attacker] {
        Some((a, src)) if src == me && crate::engine::attack::attack_def(g, a).name == "Mysterious Signal" => {}
        _ => return Ok(()),
    }
    if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
        if *prize_count > 0 {
            *prize_count += 1;
        }
    }
    Ok(())
}
