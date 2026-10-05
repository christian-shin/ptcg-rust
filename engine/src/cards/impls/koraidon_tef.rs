//! Koraidon (TEF): Primordial Beatdown — 30× for each of your Ancient Pokémon
//! in play. Shred — 130; this attack's damage isn't affected by any effects
//! on your opponent's Active Pokémon.
//!
//! Twinleaf (temporal-forces file): Primordial Beatdown sets
//! `damage = 30 × Ancient Pokémon` (Active + Bench). Shred sets
//! `ignoreDefenderEffects` (phase 4b R7B: it used to add the damage straight to
//! the Active, skipping the attacker's effects).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Koraidon@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let mut n = 0;
        for s in std::iter::once(pl.active).chain(pl.bench.iter().copied()) {
            if let Some(c) = g.st.slot_pokemon(p, s) {
                if g.st.cdef(c).has_tag(tag::ANCIENT) {
                    n += 1;
                }
            }
        }
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 30 * n;
        }
    }

    if was_attack_used(g, e, 1, me) {
        // Shred: effects on the Defending Pokémon don't change the damage.
        if let Effect::Attack { ignore_defender_effects, .. } = g.e_mut(e) {
            *ignore_defender_effects = true;
        }
    }
    Ok(())
}
