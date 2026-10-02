//! Passimian (SSP 111): Coordinated Throwing — 20 damage for each of your
//! Basic Pokémon in play (the top card of each slot).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "PassimianSSPPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let basics = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().filter(|(_, c, _)| g.st.cdef(*c).stage == Stage::Basic as u8).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 20 * basics;
        }
    }
    Ok(())
}
