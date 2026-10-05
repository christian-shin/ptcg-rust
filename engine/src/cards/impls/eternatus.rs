//! Eternatus (SSP): Dynablast — 10+; 80 more if the opponent's Active is a
//! Pokémon ex. World's End — 230; discard a Stadium in play (to its owner's
//! discard); with no Stadium the damage is set to 0.
//!
//! Fixed (phase 4b, R7F-3; ruling 1589): the Stadium was discarded in the
//! attack handler, before the damage (a Stadium that raises the target's HP
//! or reduces damage was already gone); it is now discarded in
//! AfterAttackEffect, after the damage and before the Knock Out check.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eternatus", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn move_pokemon_off_board_list(g: &mut Game, src: ListRef, dst: ListRef, source_card: CardId) -> R {
    g.run_fx(Effect::MoveCards { source: src, destination: dst, cards: None, count: None, to_top: false, to_bottom: false, skip_cleanup: false, source_card })?;
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            if let Some(c) = g.st.active_pokemon(opp as usize) {
                if g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER) {
                    if let Effect::Attack { damage, .. } = g.e_mut(e) {
                        *damage += 80;
                    }
                }
            }
        }
    }
    if was_attack_used(g, e, 1, me) {
        let has_stadium = (0..2usize).any(|q| !g.st.players[q].stadium.is_empty());
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = if has_stadium { 230 } else { 0 };
        }
    }
    if after_attack_used(g, e, 1, me) {
        if let Some(q) = (0..2usize).find(|q| !g.st.players[*q].stadium.is_empty()) {
            // MOVE_CARDS(store, state, cardList, owner.discard, { sourceCard }): whole list.
            move_pokemon_off_board_list(g, ListRef::Stadium(q as u8), ListRef::Discard(q as u8), me)?;
        }
    }
    Ok(())
}
