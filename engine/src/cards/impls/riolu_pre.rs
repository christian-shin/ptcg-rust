//! Riolu (PRE): Quick Attack — 10+; flip a coin, if heads 20 more damage.
//!
//! FLIP_A_COIN_IF_HEADS_DEAL_MORE_DAMAGE (a CoinFlipEffect whose callback
//! adds to the attack's damage after the flip's wait prompt).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Riolu@PRE", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

/// FLIP_A_COIN_IF_HEADS_DEAL_MORE_DAMAGE(store, state, effect, amount): the
/// card's `coin` fn must call [`coin_more_damage`].
pub fn flip_more_damage(g: &mut Game, me: CardId, e: EffId, amount: i32) -> R {
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    g.retain_fx(e);
    let mut f = CardFrame::at(0);
    f.e[0] = e;
    f.a[0] = amount;
    if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
        g.release_fx(e);
        return Err(err);
    }
    Ok(())
}

pub fn coin_more_damage(g: &mut Game, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    if heads {
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage += f.a[0];
        }
    }
    g.release_fx(atk);
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        flip_more_damage(g, me, e, 20)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    coin_more_damage(g, f, heads)
}
