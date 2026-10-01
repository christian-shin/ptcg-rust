//! Beldum (TEF): Dig Claws — 10. Iron Tackle — 50; this Pokémon also does
//! 10 damage to itself (THIS_POKEMON_DOES_DAMAGE_TO_ITSELF: a DealDamageEffect
//! aimed at `effect.source`).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Beldum@TEF", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        super::tapu_bulu::this_pokemon_does_damage_to_itself(g, e, 10)?;
    }
    Ok(())
}
