//! Spewpa (POR / M3): Hide — flip a coin; if heads, during your opponent's
//! next turn prevent all damage and effects from attacks done to this
//! Pokémon.
//!
//! Twinleaf quirk kept: heads only calls PREVENT_DAMAGE (a
//! PreventDamageEffect arming `preventDamageNextTurnPending = {}` on the
//! attacker's Active); effects of attacks are not prevented.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Spewpa@POR", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

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
    let r = if heads { prevent_damage(g, atk) } else { Ok(()) };
    g.release_fx(atk);
    r
}
