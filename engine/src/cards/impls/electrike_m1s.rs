//! Electrike (M1S): Thunder Jolt — 30; this Pokémon also does 10 damage to
//! itself (THIS_POKEMON_DOES_DAMAGE_TO_ITSELF).
use super::tapu_bulu::this_pokemon_does_damage_to_itself;
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Electrike@MEG", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        this_pokemon_does_damage_to_itself(g, e, 10)?;
    }
    Ok(())
}
