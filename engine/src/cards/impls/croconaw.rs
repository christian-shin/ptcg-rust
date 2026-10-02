//! Croconaw (TEF): Reverse Thrust — 30; switch this Pokémon with 1 of your
//! Benched Pokémon.
//!
//! Twinleaf: reacts to any AfterDamageEffect whose attack is Reverse Thrust
//! (SWITCH_ACTIVE_WITH_BENCHED for the effect's player).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Croconaw", mask: mask(&[k::AFTER_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::AfterDamage { b, .. } = *g.e(e) {
        if b.attack == my_attack(g, me, 0) {
            switch_active_with_benched(g, b.player as usize);
        }
    }
    Ok(())
}
