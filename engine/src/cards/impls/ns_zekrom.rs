//! N's Zekrom (M2a / ASC): Shred - 70, not affected by effects on the
//! Defending Pokémon; Rampaging Thunder - 250, can't attack next turn.
//!
//! Twinleaf (phase 4b R7B): Shred sets `ignoreDefenderEffects`; the damage goes
//! through the normal path (Weakness, Resistance and the effects on the
//! attacker apply, the effects on the Defending Pokémon don't). It used to ignore
//! Resistance on the AttackEffect and add the damage straight to the Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "NsZekrom", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { ignore_defender_effects, .. } = g.e_mut(e) {
            *ignore_defender_effects = true;
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let pl = &mut g.st.players[p as usize];
            let a = pl.active;
            pl.slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
