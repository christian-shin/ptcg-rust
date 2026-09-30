//! Skorupi (M3 / POR 51): Poison Jab — 20; the opponent's Active is now
//! Poisoned (ADD_POISON_TO_PLAYER_ACTIVE).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Skorupi", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { opp, .. } = *g.e(e) {
            add_special_conditions_to_player_active(g, opp as usize, me, &[SpecialCondition::Poisoned])?;
        }
    }
    Ok(())
}
