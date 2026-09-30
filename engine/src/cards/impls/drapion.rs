//! Drapion (M3 / POR 52): Wrack Down — 60. Hazardous Tail — 100; this
//! Pokémon also does 70 damage to itself; the opponent's Active is now
//! Paralyzed and Poisoned.
//!
//! Twinleaf: the conditions go through two AddSpecialConditionsPowerEffects
//! (ADD_POISON / ADD_PARALYZED_TO_PLAYER_ACTIVE), Poison first.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Drapion", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        let opp = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        super::tapu_bulu::this_pokemon_does_damage_to_itself(g, e, 70)?;
        add_special_conditions_to_player_active(g, opp, me, &[SpecialCondition::Poisoned])?;
        add_special_conditions_to_player_active(g, opp, me, &[SpecialCondition::Paralyzed])?;
    }
    Ok(())
}
