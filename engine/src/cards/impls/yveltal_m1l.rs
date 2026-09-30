//! Yveltal (M1L / MEG 88): Clutch — 20, the Defending Pokémon can't retreat
//! during your opponent's next turn. Dark Feather — 110.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Yveltal@Yveltal M1L", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        block_retreat(g, e)?;
    }
    Ok(())
}
