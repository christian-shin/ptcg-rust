//! Dunsparce (JTG): Trading Places — switch this Pokémon with 1 of your
//! Benched Pokémon. Pinned to the JTG card: Dunsparce TEF is a different
//! Twinleaf class with the same name.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Dunsparce@Dunsparce JTG", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        if let Effect::Attack { p, .. } = *g.e(e) {
            switch_active_with_benched(g, p as usize);
        }
    }
    Ok(())
}
