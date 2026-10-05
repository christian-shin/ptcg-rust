//! Croconaw (TEF): Reverse Thrust — 30; switch this Pokémon with 1 of your
//! Benched Pokémon.
//!
//! Twinleaf: AFTER_ATTACK for Reverse Thrust, then SWITCH_ACTIVE_WITH_BENCHED
//! for the effect's player. Fixed (R1-6): it used to react to the
//! AfterDamageEffect of the attack, so it never switched when no damage was
//! dealt (prevented, or 0 after Resistance).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Croconaw", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        if let Effect::AfterAttack { p, .. } = *g.e(e) {
            switch_active_with_benched(g, p as usize);
        }
    }
    Ok(())
}
