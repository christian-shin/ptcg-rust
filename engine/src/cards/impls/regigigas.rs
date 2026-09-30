//! Regigigas (PRE): Jewel Breaker — 100+, 230 more if your opponent's
//! Active Pokémon is a Tera Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Regigigas", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::Attack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    let tera = g.st.active_pokemon(o).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_TERA)).unwrap_or(false);
    if tera {
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 230;
        }
    }
    Ok(())
}
