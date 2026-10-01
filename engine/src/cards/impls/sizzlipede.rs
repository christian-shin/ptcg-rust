//! Sizzlipede (TEF): Heat Dive — 30; this Pokémon also does 10 damage to itself.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Sizzlipede", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        super::tapu_bulu::this_pokemon_does_damage_to_itself(g, e, 10)?;
    }
    Ok(())
}
