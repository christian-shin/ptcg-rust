//! Ceruledge ex (SSP, Tera): Abyssal Flame — 30+; 20 more damage for each
//! Energy card in your discard pile. Amethyst Rage — 280; discard all Energy
//! from this Pokémon. Tera: no attack damage while on the Bench.
//!
//! Fixed (W1-B): Amethyst Rage used to push the Energy cards of the slot
//! straight onto the discard pile and rebuild `cards` without them, leaving
//! stale references in the slot's `energies`. It now uses the
//! DISCARD_ALL_ENERGY_FROM_POKEMON prefab (a DiscardCardsEffect).
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
        super::mega_manectricex::discard_all_energy_from_pokemon(g, e, me)?;
    }

    tera_rule(g, e, me);
    Ok(())
}
