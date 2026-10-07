//! Risky Ruins (MEG): whenever any player puts a Basic non-[D] Pokémon onto
//! their Bench during their turn, put 2 damage counters on that Pokémon.
//!
//! Twinleaf checks the target slot is empty when the play effect is reduced
//! (so evolutions never trigger). Fixed in phase 4b (R4): it also requires a
//! Bench slot and the owner's own turn (a Basic put onto the Bench during the
//! opponent's turn, e.g. Dream Ball taken as a Prize card, took counters).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "DangerousRuins",
    mask: mask(&[k::PLAY_POKEMON, k::PLAY_POKEMON_FROM_DECK, k::PLAY_POKEMON_FROM_DISCARD, k::USE_STADIUM]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Automatically active: a Stadium without "that player may" can't be announced and used (Advanced Rulebook B-04).
    if let Effect::UseStadium { .. } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            bail!("CANNOT_USE_STADIUM");
        }
    }
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
    if !g.st.players[p as usize].bench.contains(&target.s) || g.st.active_player as usize != p as usize {
        return Ok(());
    }
    if d.stage == Stage::Basic as u8 {
        g.st.players[target.p as usize].slots[target.s as usize].damage += 20;
    }
    Ok(())
}

