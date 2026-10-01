//! Harlequin (WHT): each player shuffles their hand into their deck; flip a
//! coin: heads you draw 5 and your opponent draws 3, tails 3 and 5.
//!
//! Twinleaf: the Supporter moves to the supporter pile (preventDefault);
//! after the flip the player's hand (minus this card) and the opponent's
//! whole hand are moved with MoveCardsEffects (the opponent's can be
//! prevented, which skips their shuffle and draw), then SHUFFLE_DECK and
//! DRAW_CARDS for the opponent and the player.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Harlequin", mask: mask(&[k::TRAINER]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let p = match trainer_played(g, e, me) {
        Some(p) => p,
        None => return Ok(()),
    };
    if g.st.players[p].supporter_turn > 0 {
        bail!("SUPPORTER_ALREADY_PLAYED");
    }
    move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
    g.set_prevent(e, true);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let cards: Vec<CardId> = g.st.players[p].hand.iter().filter(|c| *c != me).collect();
    g.run_fx(Effect::MoveCards {
        source: ListRef::Hand(p as u8),
        destination: ListRef::Deck(p as u8),
        cards: Some(List::from_slice(&cards)),
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    let (_, prevented) = g.run_fx(Effect::MoveCards {
        source: ListRef::Hand(o as u8),
        destination: ListRef::Deck(o as u8),
        cards: None,
        count: None,
        to_top: false,
        to_bottom: false,
        skip_cleanup: false,
        source_card: me,
    })?;
    if !prevented {
        shuffle_deck(g, o);
        draw_cards(g, o, if heads { 3 } else { 5 })?;
    }
    shuffle_deck(g, p);
    draw_cards(g, p, if heads { 5 } else { 3 })?;
    Ok(())
}
