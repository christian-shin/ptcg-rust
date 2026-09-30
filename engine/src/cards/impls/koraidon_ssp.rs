//! Koraidon (SSP): Unrelenting Onslaught — 30+; 150 more if 1 of your other
//! Ancient Pokémon used an attack during your last turn. Hammer In — 110.
//!
//! BOOST_IF_OTHER_ANCIENT_ATTACKED_LAST_TURN: `ancientPokemonAttackedLastTurn`
//! and `playerLastAttack[player].sourceCard` is another (Ancient) card.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Koraidon@SSP", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if !g.st.players[p].ancient_pokemon_attacked_last_turn {
            return Ok(());
        }
        let boost = match g.st.player_last_attack[p] {
            Some((_, src)) => src != me && g.st.cdef(src).has_tag(tag::ANCIENT),
            None => false,
        };
        if boost {
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 150;
            }
        }
    }
    Ok(())
}
