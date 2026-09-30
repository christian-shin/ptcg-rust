//! Bronzong (TEF): Evolution Jammer — 30; during your opponent's next turn,
//! they can't play Pokémon from their hand to evolve. Super Psy Bolt — 100.
//!
//! Twinleaf: OPPONENT_CANNOT_EVOLVE_POKEMON is a PlayLockEffect with the
//! `evolve` flag (player-level).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Bronzong", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return opponent_cannot_play_cards(g, e, crate::effects::play_lock::EVOLVE);
    }
    Ok(())
}
