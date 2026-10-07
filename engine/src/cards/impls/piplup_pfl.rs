//! Piplup (PFL 27): Call for Support — search your deck for a Supporter
//! card, reveal it, and put it into your hand; then shuffle. Tackle — 20.
//!
//! Twinleaf: SEARCH_DECK_FOR_CARDS_TO_HAND with a `{ superType: TRAINER }`
//! filter (so the pick is shown to the opponent), min 0, max 1, and every
//! non-Supporter deck index blocked.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Piplup@PFL", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut opts = ChooseCardsOpts::new(0, 1, false);
        for (i, c) in g.st.players[p].deck.iter().enumerate() {
            let d = g.st.cdef(c);
            if !(d.is_trainer() && d.trainer_type == TrainerType::Supporter as u8) {
                opts.blocked.push(i as u8);
            }
        }
        search_deck_for_cards_to_hand(g, p, me, Filter::super_type(SuperType::Trainer), opts);
    }
    Ok(())
}
