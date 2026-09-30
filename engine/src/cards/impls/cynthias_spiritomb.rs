//! Cynthia's Spiritomb (DRI): Raging Curse - 10x the damage counters on all
//! your Benched Cynthia's Pokémon; no Weakness.
//!
//! Twinleaf: a bench slot counts when any card in its `cards` list has the
//! Cynthia's tag; `effect.damage` is set to the summed slot damage.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CynthiasSpiritomb", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Attack { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let pl = &g.st.players[p];
    let mut total = 0;
    for &b in pl.bench.iter() {
        let slot = &pl.slots[b as usize];
        if slot.cards.iter().any(|c| g.st.cdef(c).has_tag(tag::CYNTHIAS)) {
            total += slot.damage;
        }
    }
    if let Effect::Attack { damage, ignore_weakness, .. } = g.e_mut(e) {
        *damage = total;
        *ignore_weakness = true;
    }
    Ok(())
}
