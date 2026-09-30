//! Cynthia's Gible (DRI): Rock Hurl - 20; this attack's damage isn't
//! affected by Resistance.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "CynthiasGible", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { ignore_resistance, .. } = g.e_mut(e) {
            *ignore_resistance = true;
        }
    }
    Ok(())
}
