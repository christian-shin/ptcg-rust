//! Celebi (MEG 12): Traverse Time — search your deck for up to 3 in any
//! combination of [G] Pokémon and Stadium cards, reveal them, and put them
//! into your hand; then shuffle. Solar Cutter — 30.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with an empty filter and every
//! other card blocked, so the cards are never shown to the opponent.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Celebi", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut opts = ChooseCardsOpts::new(0, 3, false);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            let grass = d.is_pokemon() && d.card_type.contains(&ct::GRASS);
            let stadium = d.is_trainer() && d.trainer_type == TrainerType::Stadium as u8;
            if !grass && !stadium {
                opts.blocked.push(i as u8);
            }
        }
        search_deck_for_cards_to_hand(g, p, me, Filter::none(), opts);
    }
    Ok(())
}
