//! Haunter (SSH): Nightmare — 20; your opponent's Active Pokémon is now
//! Asleep. Spooky Shot — 40.
//!
//! Twinleaf applies the sleep on AFTER_ATTACK via
//! ADD_SLEEP_TO_PLAYER_ACTIVE (an AddSpecialConditionsPowerEffect, so it is
//! not an attack effect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Haunter", mask: mask(&[k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let o = match *g.e(e) {
        Effect::AfterAttack { opp, .. } => opp as usize,
        _ => return Ok(()),
    };
    add_special_conditions_to_player_active(g, o, me, &[SpecialCondition::Asleep])
}
