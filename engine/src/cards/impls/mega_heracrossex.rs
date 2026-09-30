//! Mega Heracross ex (PFL): Juggernaut Horn — 100+; more damage equal to the
//! damage this Pokémon took during your opponent's last turn. Mountain
//! Ramming — 170; discard the top 2 cards of your opponent's deck.
//!
//! Twinleaf: Juggernaut Horn adds the player's Active card's
//! `damageTakenLastTurn` (always defined, 0 by default).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MegaHeracrossex", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if let Some(c) = g.st.slot_pokemon(p, g.st.players[p].active) {
            let extra = g.st.cards[c as usize].damage_taken_last_turn;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += extra;
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp,
            _ => return Ok(()),
        };
        move_count_from(g, ListRef::Deck(o), ListRef::Discard(o), 2, me)?;
    }
    Ok(())
}
