//! Walrein (SSP): Frigid Fangs — 60; during your opponent's next turn,
//! Pokémon that have 2 or less Energy attached can't attack (a player-level
//! OpponentPokemonCannotAttackDuringTheirNextTurnEffect). Megaton Fall — 170;
//! this Pokémon also does 50 damage to itself.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Walrein", mask: mask(&[k::ATTACK]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        return opponent_pokemon_with_x_or_less_energy_cannot_attack(g, e, 2);
    }
    if was_attack_used(g, e, 1, me) {
        super::tapu_bulu::this_pokemon_does_damage_to_itself(g, e, 50)?;
    }
    Ok(())
}
