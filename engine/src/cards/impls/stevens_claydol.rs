//! Steven's Claydol (DRI): Eerie Light — 20; your opponent's Active Pokémon is
//! now Confused. Clay Blast — 220; discard all Energy from this Pokémon.
//!
//! Twinleaf: the confusion is applied on AFTER_ATTACK through
//! ADD_CONFUSION_TO_PLAYER_ACTIVE (an AddSpecialConditionsPowerEffect, so it is
//! not an attack effect); Clay Blast discards via DISCARD_ALL_ENERGY_FROM_POKEMON.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "StevensClaydol", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if after_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::AfterAttack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        add_special_conditions_to_player_active(g, o, me, &[SpecialCondition::Confused])?;
    }

    if was_attack_used(g, e, 1, me) {
        super::zapdos::discard_all_energy_from_active(g, e)?;
    }
    Ok(())
}
