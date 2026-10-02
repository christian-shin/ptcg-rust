//! Bronzor (SSP 126): Shield Attack — 20+; flip a coin, if heads 20 more damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BronzorSSPPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: Some(coin), can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        super::riolu_pre::flip_more_damage(g, me, e, 20)?;
    }
    Ok(())
}

fn coin(g: &mut Game, _me: CardId, f: CardFrame, heads: bool) -> R {
    super::riolu_pre::coin_more_damage(g, f, heads)
}
