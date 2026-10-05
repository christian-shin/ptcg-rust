//! Spewpa (POR / M3): Hide — flip a coin; if heads, during your opponent's
//! next turn prevent all damage and effects from attacks done to this
//! Pokémon.
//!
//! Twinleaf: FLIP_COIN_TO_PREVENT_DAMAGE_AND_EFFECTS_DURING_OPPONENTS_NEXT_TURN.
//! Phase 4b: heads used to call only PREVENT_DAMAGE; it now also calls
//! PREVENT_EFFECTS_OF_ATTACKS, like Petilil's Hide.
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
    let r = if heads { prevent_damage(g, atk).and_then(|_| prevent_effects_of_attacks(g, atk)) } else { Ok(()) };
    g.release_fx(atk);
    r
}
