//! Risky Ruins (MEG): whenever any player puts a Basic non-[D] Pokémon onto
//! their Bench, put 2 damage counters on that Pokémon.
//!
//! Twinleaf checks the target slot is empty when the play effect is reduced
//! (so evolutions never trigger); it does not check that the slot is a
//! Bench slot.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "DangerousRuins",
    mask: mask(&[k::PLAY_POKEMON, k::PLAY_POKEMON_FROM_DECK, k::PLAY_POKEMON_FROM_DISCARD]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (p, card, target) = match *g.e(e) {
        Effect::PlayPokemon { p, card, target, .. } => (p, card, target),
        Effect::PlayPokemonFromDeck { p, card, target } => (p, card, target),
        Effect::PlayPokemonFromDiscard { p, card, target } => (p, card, target),
        _ => return Ok(()),
    };
    if g.st.stadium_card() != Some(me) {
        return Ok(());
    }
    if is_stadium_effect_blocked(g, p as usize, target, me) {
        return Ok(());
    }
    let d = g.st.cdef(card);
    if !g.st.slot(target.p as usize, target.s).cards.is_empty() || d.card_type.contains(&ct::DARK) {
        return Ok(());
    }
    if d.stage == Stage::Basic as u8 {
        g.st.players[target.p as usize].slots[target.s as usize].damage += 20;
    }
    Ok(())
}

