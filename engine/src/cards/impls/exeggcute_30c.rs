//! Exeggcute (30C): Hypnosis — your opponent's Active Pokémon is now Asleep.
//!
//! Twinleaf has two `Exeggcute` classes; this port is bound to 30C.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Exeggcute@30C", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        add_special_conditions_to_opponent_active(g, e, &[SpecialCondition::Asleep])?;
    }
    Ok(())
}
