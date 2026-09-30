//! Moltres (M2 / PFL): Fighting Wings — 20+, 90 more if the opponent's Active
//! Pokémon is a Pokémon ex.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Moltres", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        // `hasTag(CardTag.POKEMON_ex)` (runtime tags: not modeled).
        if let Some(c) = g.st.active_pokemon(opp) {
            if g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER) {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 90;
                }
            }
        }
    }
    Ok(())
}
