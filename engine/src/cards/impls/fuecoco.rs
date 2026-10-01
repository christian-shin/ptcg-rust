//! Fuecoco (SSP): Heat Burn — 20; the opponent's Active Pokémon is now Burned.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Fuecoco", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Burned])?;
    }
    Ok(())
}
