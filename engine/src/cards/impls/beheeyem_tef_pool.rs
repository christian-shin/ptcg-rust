//! Beheeyem (TEF 74): Cosmic Beatdown — 20 damage for each of your Pokémon in
//! play (one per occupied slot).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BeheeyemTEFPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = for_each_pokemon(g, p, PlayerType::BottomPlayer).len() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 20 * n;
        }
    }
    Ok(())
}
