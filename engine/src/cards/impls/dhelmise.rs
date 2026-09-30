//! Dhelmise (PBL / M5): Vengeful Anchor — 30+, 140 more if you have 4 or
//! more Pokémon with the Hide 'n' Sneak Ability in your discard pile.
use super::shuppet::count_hide_n_sneak_in_discard;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dhelmise", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            if count_hide_n_sneak_in_discard(g, p as usize) >= 4 {
                if let Effect::Attack { damage, .. } = g.e_mut(e) {
                    *damage += 140;
                }
            }
        }
    }
    Ok(())
}
