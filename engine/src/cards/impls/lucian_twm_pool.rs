//! Lucian (TWM 157, Supporter): each player shuffles their hand and puts it
//! on the bottom of their deck. If either player put any cards on the bottom
//! of their deck in this way, each player flips a coin. If heads, that player
//! draws 6 cards. If tails, they draw 3 cards.
//!
//! Twinleaf: throws SUPPORTER_ALREADY_PLAYED, then CANNOT_PLAY_THIS_CARD when
//! both hands (excluding this card) are empty. Each hand is permuted with
//! `Chance.shuffle(n)` (player first, then opponent) and moved to the end of
//! the deck by MOVE_CARDS (empty hands skip both the shuffle and the move).
//! Then the player flips, then the opponent (in the first flip's callback);
//! both draw afterwards (player first). Note the coin flips happen even though
//! the text only requires them when cards were moved; the throw above makes
//! that always true.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "LucianTWMPool", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: Some(coin), can_play: None };

fn shuffled(g: &mut Game, cards: &[CardId]) -> Vec<CardId> {
    let n = cards.len();
    let mut perm = [0u8; 120];
    g.rng.shuffle(n, &mut perm);
    (0..n).map(|i| cards[perm[i] as usize]).collect()
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    let o = 1 - p;
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    let player_cards: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    let opponent_cards: Vec<CardId> = g.st.players[o].hand.iter().collect();
    if player_cards.is_empty() && opponent_cards.is_empty() {
        bail!("CANNOT_PLAY_THIS_CARD");
    }
    if !player_cards.is_empty() {
        let s = shuffled(g, &player_cards);
        move_cards(g, ListRef::Hand(p as u8), ListRef::Deck(p as u8), &s, me)?;
    }
    if !opponent_cards.is_empty() {
        let s = shuffled(g, &opponent_cards);
        move_cards(g, ListRef::Hand(o as u8), ListRef::Deck(o as u8), &s, me)?;
    }
    let mut f = CardFrame::at(0);
    f.a[0] = p as i32;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        0 => {
            let mut nf = CardFrame::at(1);
            nf.a[0] = p as i32;
            nf.a[1] = heads as i32;
            g.coin_flip(1 - p, CoinCb::Card { card: me, frame: nf })?;
            Ok(())
        }
        _ => {
            let player_heads = f.a[1] != 0;
            draw_cards(g, p, if player_heads { 6 } else { 3 })?;
            draw_cards(g, 1 - p, if heads { 6 } else { 3 })
        }
    }
}
