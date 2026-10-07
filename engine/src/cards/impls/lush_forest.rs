//! Forest of Vitality (MEG, stadium): each player's [G] Pokémon can evolve
//! into [G] Pokémon during the turn they play those Pokémon, except during
//! their first turn.
//!
//! Twinleaf: when a [G] Pokémon card is played (PlayPokemonEffect, reduced
//! before the core places it), every [G] Pokémon already in play for that
//! player (CheckPokemonTypeEffect) gets `pokemonPlayedTurn = turn - 1`. A
//! newly benched Basic is not in play yet, so it is only reset by a later
//! [G] play that turn.
use crate::cards::prelude::*;
use crate::engine::game_effect::pokemon_types;

pub static IMPL: CardImpl = CardImpl { class: "LushForest", mask: mask(&[k::PLAY_POKEMON, k::USE_STADIUM]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Automatically active: a Stadium without "that player may" can't be announced and used (Advanced Rulebook B-04).
    if let Effect::UseStadium { .. } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            bail!("CANNOT_USE_STADIUM");
        }
    }
    let (p, card, target) = match *g.e(e) {
        Effect::PlayPokemon { p, card, target, .. } => (p as usize, card, target),
        _ => return Ok(()),
    };
    if g.st.stadium_card() != Some(me) {
        return Ok(());
    }
    if g.st.turn <= 2 || !g.st.cdef(card).card_type.contains(&ct::GRASS) {
        return Ok(());
    }
    if is_stadium_effect_blocked(g, p, target, me) {
        return Ok(());
    }
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        let sr = SlotRef::new(p, s);
        if is_stadium_effect_blocked(g, p, sr, me) {
            continue;
        }
        let (t, _) = g.run_fx(Effect::CheckPokemonType { target: sr, card_types: pokemon_types(g, sr) })?;
        let grass = match t {
            Effect::CheckPokemonType { card_types, .. } => card_types.contains(&ct::GRASS),
            _ => false,
        };
        if grass {
            g.st.players[p].slots[s as usize].pokemon_played_turn = g.st.turn as i32 - 1;
        }
    }
    Ok(())
}
