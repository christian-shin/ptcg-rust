//! Search support (PLAN.md 3.5): determinization of hidden information.
//!
//! From one player's point of view the hidden cards are their own deck order
//! and face-down prizes, and the opponent's hand, deck and prizes. A
//! determinization re-deals those cards among those positions, keeping every
//! zone's size, so a search can roll out a fully specified game.

use crate::game::{Game, GameError, R};
use crate::list::*;
use crate::rng::Rng;

/// Hidden cards of `owner` as seen by `viewer`, and their zones.
fn hidden_zones(viewer: usize, owner: usize) -> (bool, bool) {
    // (hand hidden, deck+prizes hidden)
    (viewer != owner, true)
}

/// Assign hidden cards explicitly. `deck` gives the new deck order, `prizes`
/// the contents of each prize slot, `hand` the opponent's hand (ignored for
/// the viewer's own hand). The multiset of hidden cards must be unchanged.
pub fn set_hidden(g: &mut Game, viewer: usize, owner: usize, deck: &[CardId], prizes: &[Vec<CardId>], hand: Option<&[CardId]>) -> R {
    let (hand_hidden, _) = hidden_zones(viewer, owner);
    let pl = &g.st.players[owner];
    let mut before: Vec<CardId> = pl.deck.iter().collect();
    for i in 0..pl.prize_count as usize {
        before.extend(pl.prizes[i].iter());
    }
    if hand_hidden {
        before.extend(pl.hand.iter());
    }
    let mut after: Vec<CardId> = deck.to_vec();
    for p in prizes {
        after.extend(p.iter().copied());
    }
    if hand_hidden {
        after.extend(hand.ok_or(GameError("HAND_REQUIRED"))?.iter().copied());
    }
    let (mut a, mut b) = (before.clone(), after.clone());
    a.sort();
    b.sort();
    if a != b {
        return Err(GameError("HIDDEN_CARDS_MISMATCH"));
    }
    let pl = &mut g.st.players[owner];
    if deck.len() != pl.deck.len() || prizes.len() != pl.prize_count as usize {
        return Err(GameError("ZONE_SIZE_MISMATCH"));
    }
    for (i, p) in prizes.iter().enumerate() {
        if p.len() != pl.prizes[i].len() {
            return Err(GameError("ZONE_SIZE_MISMATCH"));
        }
    }
    pl.deck.set_from(deck);
    for (i, p) in prizes.iter().enumerate() {
        pl.prizes[i].set_from(p);
    }
    if hand_hidden {
        let h = hand.unwrap();
        if h.len() != pl.hand.len() {
            return Err(GameError("ZONE_SIZE_MISMATCH"));
        }
        pl.hand.set_from(h);
    }
    Ok(())
}

/// Uniformly random determinization for `viewer`, and a fresh chance seed.
pub fn determinize(g: &mut Game, viewer: usize, seed: u32) -> R {
    let mut rng = Rng::new(seed);
    for owner in 0..2 {
        let (hand_hidden, _) = hidden_zones(viewer, owner);
        let pl = &g.st.players[owner];
        let mut pool: Vec<CardId> = pl.deck.iter().collect();
        let prize_sizes: Vec<usize> = (0..pl.prize_count as usize).map(|i| pl.prizes[i].len()).collect();
        for i in 0..pl.prize_count as usize {
            pool.extend(pl.prizes[i].iter());
        }
        let hand_len = pl.hand.len();
        if hand_hidden {
            pool.extend(pl.hand.iter());
        }
        let deck_len = pl.deck.len();
        for i in (1..pool.len()).rev() {
            pool.swap(i, rng.index(i + 1));
        }
        let mut it = pool.into_iter();
        let deck: Vec<CardId> = (&mut it).take(deck_len).collect();
        let prizes: Vec<Vec<CardId>> = prize_sizes.iter().map(|n| (&mut it).take(*n).collect()).collect();
        let hand: Vec<CardId> = if hand_hidden { (&mut it).take(hand_len).collect() } else { vec![] };
        set_hidden(g, viewer, owner, &deck, &prizes, if hand_hidden { Some(&hand) } else { None })?;
    }
    g.rng = Rng::new(rng.next_u32());
    Ok(())
}
