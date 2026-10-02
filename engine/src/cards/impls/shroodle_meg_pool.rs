//! Shroodle (MEG 91): Poison Jab — 20; your opponent's Active Pokémon is now
//! Poisoned (an AddSpecialConditionsEffect [POISONED]).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "ShroodleMEGPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Poisoned])?;
    }
    Ok(())
}
