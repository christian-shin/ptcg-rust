//! Brute Bonnet (TWM): Poison Spray — the opponent's Active is now Poisoned.
//! Relentless Punches — 50+, 50 more for each damage counter on the
//! opponent's Active.
//!
//! Twinleaf: Poison Spray uses ADD_POISON_TO_PLAYER_ACTIVE, an
//! AddSpecialConditionsPowerEffect (not the attack effect), which also sets
//! the target's poison/burn/sleep/confusion values to the defaults.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "BruteBonnet", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            add_special_conditions_to_player_active(g, opp as usize, me, &[SpecialCondition::Poisoned])?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            let o = opp as usize;
            let d = g.st.slot(o, g.st.players[o].active).damage;
            if let Effect::Attack { damage, .. } = g.e_mut(e) {
                *damage += 5 * d;
            }
        }
    }
    Ok(())
}
