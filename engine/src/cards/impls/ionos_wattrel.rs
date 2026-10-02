//! Iono's Wattrel (JTG / ASC): Quick Attack - 10+; flip a coin, if heads this
//! attack does 20 more damage.
//!
//! Twinleaf: COIN_FLIP_PROMPT with `effect.damage += 20` in the callback (the
//! attack effect is retained across the flip).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "IonosWattrel", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        super::riolu_pre::flip_more_damage(g, me, e, 20)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    super::riolu_pre::coin_more_damage(g, f, heads)
}
