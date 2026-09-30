//! Toucannon (M5): Aerial Draw — once during your turn, draw a card.
//! Feather Rondo — 60+, 20 more for each Benched Pokémon (both sides).
//!
//! Twinleaf: IS_ABILITY_BLOCKED → BLOCKED_BY_EFFECT, then
//! USE_ABILITY_ONCE_PER_TURN, then an empty deck throws CANNOT_USE_POWER.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Toucannon",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN, k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn sky_draw() -> crate::markers::MarkerName {
    crate::marker!("M5_TOUCANNON_SKYDRAW")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(sky_draw(), me);
        }
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if is_ability_blocked(g, p, me, None) {
            bail!("BLOCKED_BY_EFFECT");
        }
        use_ability_once_per_turn(g, p, sky_draw(), me)?;
        if g.st.players[p].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        draw_cards(g, p, 1)?;
        ability_used(g, p, me);
    }
    remove_marker_at_end_of_turn(g, e, sky_draw(), me);
    if was_attack_used(g, e, 0, me) {
        let (p, o) = match *g.e(e) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => return Ok(()),
        };
        let count = |q: usize| {
            let pl = &g.st.players[q];
            pl.bench.iter().filter(|b| !pl.slots[**b as usize].cards.is_empty()).count() as i32
        };
        let n = count(p) + count(o);
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage += 20 * n;
        }
    }
    Ok(())
}
