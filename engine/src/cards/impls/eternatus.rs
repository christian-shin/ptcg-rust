//! Eternatus (SSP): Dynablast — 10+; 80 more if the opponent's Active is a
//! Pokémon ex. World's End — 230; discard a Stadium in play (to its owner's
//! discard); with no Stadium the damage is set to 0.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Eternatus", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

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
        let stadium = (0..2usize).find(|q| !g.st.players[*q].stadium.is_empty());
        match stadium {
            None => {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage = 0;
                }
            }
            Some(q) => {
                // MOVE_CARDS(store, state, cardList, owner.discard, { sourceCard }): whole list.
                move_pokemon_off_board_list(g, ListRef::Stadium(q as u8), ListRef::Discard(q as u8), me)?;
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage = 230;
                }
            }
        }
    }
    Ok(())
}
