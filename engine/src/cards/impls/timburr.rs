//! Timburr (TWM 103): Best Punch — 40; flip a coin, if tails this attack
//! does nothing (`effect.damage = 0`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Timburr", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    g.retain_fx(e);
    let mut f = CardFrame::at(0);
    f.e[0] = e;
    g.coin_flip(p, CoinCb::Card { card: me, frame: f })?;
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    if !heads {
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 0;
        }
    }
    g.release_fx(atk);
    Ok(())
}
