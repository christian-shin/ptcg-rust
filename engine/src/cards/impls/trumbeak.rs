//! Trumbeak (M5): Fly — 30; flip a coin. If tails, this attack does nothing.
//! If heads, during your opponent's next turn, prevent all damage from and
//! effects of attacks done to this Pokémon.
//!
//! Twinleaf FLIP_COIN_FOR_FLY: tails sets the attack damage to 0; heads
//! reduces a PreventDamageEffect and a PreventEffectsOfAttacksEffect (both
//! with the empty filter, target = the attacker's slot).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Trumbeak", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
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
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    let atk = f.e[0];
    let r = if !heads {
        if let Effect::Attack { damage, .. } = g.e_mut(atk) {
            *damage = 0;
        }
        Ok(())
    } else {
        prevent_damage(g, atk).and_then(|_| prevent_effects_of_attacks(g, atk))
    };
    g.release_fx(atk);
    r
}
