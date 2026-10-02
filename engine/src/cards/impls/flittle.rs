//! Flittle (SSP): Splashing Dodge - 10; flip a coin, if heads, during your
//! opponent's next turn, prevent all damage from and effects of attacks done
//! to this Pokémon.
//!
//! Twinleaf: PREVENT_DAMAGE then PREVENT_EFFECTS_OF_ATTACKS on heads.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Flittle", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
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
    if heads {
        if let Err(err) = prevent_damage(g, atk).and_then(|_| prevent_effects_of_attacks(g, atk)) {
            g.release_fx(atk);
            return Err(err);
        }
    }
    g.release_fx(atk);
    Ok(())
}
