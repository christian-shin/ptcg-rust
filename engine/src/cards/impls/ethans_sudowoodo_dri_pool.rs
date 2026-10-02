//! Ethan's Sudowoodo (DRI 93): Impound — 20; during your opponent's next turn
//! the Defending Pokémon can't retreat. Try to Imitate — flip a coin; if
//! heads, choose 1 of your opponent's Active Pokémon's attacks and use it as
//! this attack.
//!
//! Twinleaf: BLOCK_RETREAT; COIN_FLIP_PROMPT then
//! COPY_OPPONENT_ACTIVE_ATTACK_WITH_RETRY (COPY_ATTACK_FROM_POKEMON_LIST with
//! maxRetries 3, see `copy_attack.rs`). The attack effect is retained across
//! the coin flip.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "EthansSudowoodoDRIPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        block_retreat(g, e)?;
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(1);
        f.e[0] = e;
        if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    let r = if heads { crate::copy_attack::copy_opponent_active_attack_with_retry(g, atk) } else { Ok(()) };
    g.release_fx(atk);
    r
}
