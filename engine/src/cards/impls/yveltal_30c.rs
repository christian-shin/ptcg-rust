//! Yveltal (30C 100): Life-Locked — your opponent's Active Pokémon can't be
//! healed. Dark Cutter — 90.
//!
//! Fixed in phase 4b (R4): the HealEffect handler loops over `state.players`
//! and skips a player who does not have this card in play (it used to return
//! at the first such player, so the Ability only worked when this card
//! belonged to the first player, index 0). A blocked Ability skips that
//! player's turn of the loop too.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Yveltal@30C", mask: mask(&[k::HEAL]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let target = match *g.e(e) {
        Effect::Heal { target, .. } => target,
        _ => return Ok(()),
    };
    for p in 0..2 {
        let in_play = for_each_pokemon_cards_contains(g, p, me);
        if !in_play {
            continue;
        }
        if is_ability_blocked(g, p, me, None) {
            continue;
        }
        let o = 1 - p;
        if target.p as usize == o && target.s == g.st.players[o].active {
            g.set_prevent(e, true);
            return Ok(());
        }
    }
    Ok(())
}

/// `StateUtils.isPokemonInPlay(player, card)`: the card is the Pokémon card
/// of one of the player's in-play slots.
fn for_each_pokemon_cards_contains(g: &Game, p: usize, c: CardId) -> bool {
    for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().any(|(_, x, _)| *x == c)
}
