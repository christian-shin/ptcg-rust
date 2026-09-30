//! Illumise (TWM): Slowing Perfume — only if you go second, during your
//! first turn: shuffle 1 of your opponent's Benched Pokémon and all attached
//! cards into their deck. Glide — 30.
//!
//! Twinleaf quirks kept: legal only on `state.turn == 2` (throws
//! CANNOT_USE_ATTACK otherwise), and it moves the opponent's *Active*
//! Pokémon stack (not a Benched one) into their deck, runs `clearEffects()`
//! on the vacated Active slot, then prompts the opponent's deck shuffle.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Illumise@TWM", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    if g.st.turn != 2 {
        bail!("CANNOT_USE_ATTACK");
    }
    let o = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let a = g.st.players[o].active;
    move_pokemon_off_board(g, SlotRef::new(o, a), ListRef::Deck(o as u8), me)?;
    let a = g.st.players[o].active;
    crate::engine::game_effect::clear_effects(&mut g.st.players[o].slots[a as usize]);
    shuffle_deck(g, o);
    Ok(())
}
