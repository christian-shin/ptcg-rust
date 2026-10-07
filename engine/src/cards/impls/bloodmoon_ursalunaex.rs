//! Bloodmoon Ursaluna ex (TWM): Seasoned Skill — Blood Moon costs [C] less
//! for each Prize card your opponent has taken. Blood Moon — 240; during
//! your next turn, this Pokémon can't attack.
//!
//! Twinleaf: the reduction runs on CheckAttackCostEffect for this card's
//! attack (after the ability-lock probe, before the [C] check), removing
//! 6 - prizesLeft [C] (none when the opponent has 6 or 0 prizes left).
//! Blood Moon sets `cannotAttackNextTurnPending` on the player's Active.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "BloodmoonUrsalunaex",
    mask: mask(&[k::CHECK_ATTACK_COST, k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::CheckAttackCost { p, attack, .. } = *g.e(e) {
        if attack == my_attack(g, me, 0) {
            let p = p as usize;
            if is_ability_blocked(g, p, me, None) {
                return Ok(());
            }
            let remaining = g.st.players[1 - p].prize_left();
            let n = match remaining {
                1..=5 => 6 - remaining,
                _ => 0,
            };
            // Applied once, with the other cost changes, after all handlers ran (D-11, D-12).
            if let Effect::CheckAttackCost { reduction, .. } = g.e_mut(e) {
                *reduction = reduction.saturating_add(n as u8);
            }
            return Ok(());
        }
    }
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            let p = p as usize;
            let a = g.st.players[p].active;
            g.st.players[p].slots[a as usize].cannot_attack_next_turn_pending = true;
        }
    }
    Ok(())
}
