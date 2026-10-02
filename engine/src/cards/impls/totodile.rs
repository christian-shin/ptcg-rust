//! Totodile (TEF): Big Bite — 10; during your opponent's next turn, the
//! Defending Pokémon can't retreat (BLOCK_RETREAT: a PreventRetreatEffect).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Totodile", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        block_retreat(g, e)?;
    }
    Ok(())
}
