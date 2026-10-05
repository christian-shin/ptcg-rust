//! Skorupi (M3 / POR 51): Poison Jab — 20; the opponent's Active is now
//! Poisoned (YOUR_OPPPONENTS_ACTIVE_POKEMON_IS_NOW_POISIONED: an
//! AddSpecialConditionsEffect; fixed in phase 4b, R4: it was an Ability-style
//! AddSpecialConditionsPowerEffect via ADD_POISON_TO_PLAYER_ACTIVE).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Skorupi", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Poisoned])?;
    }
    Ok(())
}
