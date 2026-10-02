//! Clefable (TWM / ASC): Metronome - choose 1 of your opponent's Active
//! Pokémon's attacks and use it as this attack (COPY_OPPONENT_ACTIVE_ATTACK_WITH_RETRY,
//! see `copy_attack.rs`). Magical Shot - 100.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Clefable@TWM|ASC", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return crate::copy_attack::copy_opponent_active_attack_with_retry(g, e);
    }
    Ok(())
}
