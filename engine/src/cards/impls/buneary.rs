//! Buneary (PFL / M2): Run Around — switch this Pokémon with 1 of your
//! Benched Pokémon (AFTER_ATTACK). Kick — 20.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Buneary", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            switch_active_with_benched(g, p as usize);
        }
    }
    Ok(())
}
