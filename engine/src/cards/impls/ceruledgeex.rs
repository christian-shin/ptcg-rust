//! Ceruledge ex (SSP, Tera): Abyssal Flame — 30+; 20 more damage for each
//! Energy card in your discard pile. Amethyst Rage — 280; discard all Energy
//! from this Pokémon. Tera: no attack damage while on the Bench.
//!
//! Twinleaf: Amethyst Rage pushes the Energy cards of the slot holding this
//! card straight onto the discard pile and rebuilds `cards` without them, so
//! no MoveCardsEffect runs and the slot's `energies` list keeps the stale
//! references (ported as is).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Ceruledgeex", mask: mask(&[k::ATTACK, k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[p].discard.iter().filter(|c| g.st.cdef(*c).is_energy()).count() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += n * 20;
        }
    }

    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if let Some((sp, ss)) = g.st.find_pokemon_slot(me) {
            let energy: Vec<CardId> = g.st.slot(sp, ss).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).collect();
            for c in energy.iter() {
                g.lst_mut(ListRef::Discard(p as u8)).push(*c);
            }
            let rest: Vec<CardId> = g.st.slot(sp, ss).cards.iter().filter(|c| !g.st.cdef(*c).is_energy()).collect();
            g.st.players[sp].slots[ss as usize].cards.set_from(&rest);
        }
    }

    tera_rule(g, e, me);
    Ok(())
}
