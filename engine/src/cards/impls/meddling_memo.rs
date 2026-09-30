//! Meddling Memo (SSP): your opponent counts the cards in their hand,
//! shuffles them, and puts them on the bottom of their deck. If they do,
//! they draw that many cards.
//!
//! Twinleaf: the hand is permuted in place with `Chance.shuffle(n)` (drawn
//! from the game RNG, no prompt), moved to the end of the deck, then the
//! same count is moved deck→hand from the top; the card then moves
//! supporter→discard.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "MeddlingMemo", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    let (pu, ou) = (p as u8, o as u8);
    if g.st.players[o].hand.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    g.set_prevent(e, true);
    move_cards(g, ListRef::Hand(pu), ListRef::Supporter(pu), &[me], me)?;
    let n = g.st.players[o].hand.len();
    let mut perm = [0u8; 120];
    g.rng.shuffle(n, &mut perm);
    let copy: Vec<CardId> = g.st.players[o].hand.as_slice().to_vec();
    {
        let hand = g.st.players[o].hand.as_mut_slice();
        for i in 0..n {
            hand[i] = copy[perm[i] as usize];
        }
    }
    g.run_fx(Effect::MoveCards {
        source: ListRef::Hand(ou),
        destination: ListRef::Deck(ou),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    move_count_from(g, ListRef::Deck(ou), ListRef::Hand(ou), n, me)?;
    move_cards(g, ListRef::Supporter(pu), ListRef::Discard(pu), &[me], me)
}
