//! Drapion (M3 / POR 52): Wrack Down — 60. Hazardous Tail — 100; this
//! Pokémon also does 70 damage to itself; the opponent's Active is now
//! Paralyzed and Poisoned.
//!
//! Twinleaf: the conditions go through two AddSpecialConditionsEffects
//! (YOUR_OPPPONENTS_ACTIVE_POKEMON_IS_NOW_POISIONED / _PARALYZED), Poison
//! first. Fixed in phase 4b (R4): they were AddSpecialConditionsPowerEffects
//! (ADD_POISON / ADD_PARALYZED_TO_PLAYER_ACTIVE), an Ability-style effect that
//! Mist Energy and effect prevention don't stop.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Drapion", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        super::tapu_bulu::this_pokemon_does_damage_to_itself(g, e, 70)?;
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Poisoned])?;
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Paralyzed])?;
    }
    Ok(())
}
