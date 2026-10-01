//! Snorunt (TWM): Astonish — 20; choose a random card from your opponent's
//! hand (`Chance.index`), your opponent reveals it (ShowCardsPrompt for the
//! attacker) and shuffles it into their deck (MOVE_CARD_TO, no effect;
//! SHUFFLE_DECK for the opponent).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Snorunt", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let n = g.st.players[o].hand.len();
        if n > 0 {
            let i = g.rng.index(n);
            let c = g.st.players[o].hand.as_slice()[i];
            show_cards_to_player(g, p, 1);
            g.move_card_to(ListRef::Hand(o as u8), c, ListRef::Deck(o as u8));
            shuffle_deck(g, o);
        }
    }
    Ok(())
}
