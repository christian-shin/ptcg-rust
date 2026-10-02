//! Swablu (SSP 148): Disarming Voice — 10; your opponent's Active Pokémon is
//! now Confused (AddSpecialConditionsEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SwabluSSPPool", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Confused])?;
    }
    Ok(())
}
