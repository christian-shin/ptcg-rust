//! Ho-Oh (SSP): Flap — 50. Shining Blaze — 100+; 100 more if you have a Tera
//! Pokémon in play.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "HoOh", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let tera = for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, c, _)| g.st.cdef(*c).has_tag(tag::POKEMON_TERA));
        if tera {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 100;
            }
        }
    }
    Ok(())
}
