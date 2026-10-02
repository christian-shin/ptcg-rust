//! Chandelure (TWM 38): Alluring Light — once during your turn, you may have
//! each player draw a card. Mind Ruler — 30 damage for each card in your
//! opponent's hand.
//!
//! Twinleaf: the marker is removed on PlayPokemon of this card and at the end
//! of each turn; the ability throws BLOCKED_BY_EFFECT when blocked,
//! POWER_ALREADY_USED with the marker, CANNOT_USE_POWER with both decks empty;
//! then marker + ABILITY_USED, you draw 1, then your opponent draws 1.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "ChandelureTWMPool",
    mask: mask(&[k::PLAY_POKEMON, k::POWER, k::END_TURN, k::ATTACK]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn mk() -> crate::markers::MarkerName {
    marker!("TWM_CHANDELURE_ALLURING_LIGHT")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PlayPokemon { p, card, .. } = *g.e(e) {
        if card == me {
            g.st.players[p as usize].marker.remove_from(mk(), me);
        }
    }
    if was_power_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        if is_ability_blocked(g, p, me, None) {
            bail!("BLOCKED_BY_EFFECT");
        }
        if g.st.players[p].marker.has_from(mk(), me) {
            bail!("POWER_ALREADY_USED");
        }
        if g.st.players[p].deck.is_empty() && g.st.players[o].deck.is_empty() {
            bail!("CANNOT_USE_POWER");
        }
        use_ability_once_per_turn(g, p, mk(), me)?;
        ability_used(g, p, me);
        draw_cards(g, p, 1)?;
        draw_cards(g, o, 1)?;
    }
    remove_marker_at_end_of_turn(g, e, mk(), me);
    if was_attack_used(g, e, 0, me) {
        let o = match *g.e(e) {
            Effect::Attack { opp, .. } => opp as usize,
            _ => return Ok(()),
        };
        let n = g.st.players[o].hand.len() as i32;
        if let Effect::Attack { damage, .. } = g.e_mut(e) {
            *damage = 30 * n;
        }
    }
    Ok(())
}
