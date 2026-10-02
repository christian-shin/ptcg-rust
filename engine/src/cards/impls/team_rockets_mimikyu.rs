//! Team Rocket's Mimikyu (DRI 87 / ASC): Gemstone Hunt — choose an attack on
//! your opponent's Active Tera Pokémon and use it as the effect of this
//! attack.
//!
//! Twinleaf: nothing unless the opponent's Active has a Pokémon card with
//! attacks and the Tera tag; then COPY_ATTACK_FROM_POKEMON_LIST with that
//! card (allowCancel false, maxRetries 1, see `copy_attack.rs`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "TeamRocketsMimikyu", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let opp = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let oa = g.st.players[opp].active;
    let card = match g.st.slot_pokemon(opp, oa) {
        Some(c) if !g.st.cdef(c).attacks.is_empty() => c,
        _ => return Ok(()),
    };
    if !g.st.cdef(card).has_tag(tag::POKEMON_TERA) {
        return Ok(());
    }
    crate::copy_attack::copy_attack_from_pokemon_list(g, e, &[card], false)
}
