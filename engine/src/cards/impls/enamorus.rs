//! Enamorus (TWM): Heart Sign - 30. Love Resonance - 80+; 120 more if any of
//! your Pokémon in play share a type with any of your opponent's.
//!
//! Twinleaf reads the types of `cardList.cards[0]` (the first card of each
//! in-play slot, i.e. the bottom Pokémon), not the top evolution.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Enamorus", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn types_in_play(g: &Game, p: usize, pt: PlayerType) -> Vec<CardType> {
    let mut out: Vec<CardType> = Vec::new();
    for (s, _, _) in for_each_pokemon(g, p, pt).iter() {
        if let Some(c) = g.st.slot(p, *s).cards.iter().next() {
            for &t in g.st.cdef(c).card_type {
                if !out.contains(&t) {
                    out.push(t);
                }
            }
        }
    }
    out
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mine = types_in_play(g, p, PlayerType::BottomPlayer);
        let theirs = types_in_play(g, 1 - p, PlayerType::TopPlayer);
        if mine.iter().any(|t| theirs.contains(t)) {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 120;
            }
        }
    }
    Ok(())
}
