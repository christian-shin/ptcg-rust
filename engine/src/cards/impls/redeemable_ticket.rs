//! Redeemable Ticket (JTG): count your Prize cards and shuffle them face
//! down, then put them at the bottom of your deck. If you do, add that many
//! cards from the top of your deck to your Prize cards.
//!
//! Twinleaf quirks kept: the card MOVE_CARDS hand→discard (a no-op: the item
//! is already in the supporter pile) and later supporter→discard. The prize
//! cards are permuted with `Chance.shuffle(n)` (game RNG, no prompt), then
//! each is `unshift`ed onto the deck — i.e. put on TOP, in reverse order —
//! and the new prizes are `pop`ped from the BOTTOM of the deck into the
//! first empty prize slots. The deck is not shuffled.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "RedeemableTicket", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let pc = g.st.players[p].prize_count as usize;
    let count: usize = (0..pc).map(|i| g.st.players[p].prizes[i].len()).sum();
    if count == 0 {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    let mut all: Vec<CardId> = Vec::new();
    for i in 0..pc {
        all.extend(g.st.players[p].prizes[i].iter());
    }
    let mut perm = [0u8; 120];
    g.rng.shuffle(all.len(), &mut perm);
    let copy = all.clone();
    for i in 0..all.len() {
        all[i] = copy[perm[i] as usize];
    }
    // unshift each card: the last one ends up on top.
    let mut deck: Vec<CardId> = all.iter().rev().copied().collect();
    deck.extend(g.st.players[p].deck.iter());
    for i in 0..pc {
        g.st.players[p].prizes[i].set_from(&[]);
    }
    for _ in 0..count {
        let c = match deck.pop() {
            Some(c) => c,
            None => continue,
        };
        match (0..pc).find(|i| g.st.players[p].prizes[*i].is_empty()) {
            Some(i) => g.st.players[p].prizes[i].set_from(&[c]),
            None => deck.push(c),
        }
    }
    g.st.players[p].deck.set_from(&deck);
    move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)
}
