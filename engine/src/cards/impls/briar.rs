//! Briar (SCR): usable only if the opponent has exactly 2 Prize cards left;
//! this turn, if the opponent's Active is Knocked Out by damage from an attack
//! of your Tera Pokémon, take 1 more Prize card.
//!
//! Twinleaf keeps the flag on the card instance (`extraPrizes`); every Briar
//! copy in any zone reacts to Active knock-outs during the attack phase.
//! Fixed in phase 4b (R4): the flag is cleared at every end of turn ("during
//! this turn"); it used to survive until the next knock-out of an Active
//! Pokémon, so it could apply on a later turn or to the opponent.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Briar", mask: mask(&[k::TRAINER, k::KNOCK_OUT, k::END_TURN]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { .. } = *g.e(e) {
        g.st.cards[me as usize].extra_prizes = false;
    }
    if let Some(p) = trainer_played(g, e, me) {
        let o = 1 - p;
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        if g.st.players[o].prize_left() != 2 {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.st.cards[me as usize].extra_prizes = true;
        return Ok(());
    }
    if let Effect::KnockOut { p, target, prize_count, .. } = *g.e(e) {
        let p = p as usize;
        if target.p as usize != p || target.s != g.st.players[p].active {
            return Ok(());
        }
        let o = 1 - p;
        if g.st.phase != GamePhase::Attack || g.st.active_player as usize != o {
            return Ok(());
        }
        // The Pokémon that used the attack, wherever it is by now (fixed in phase 4b, F1: it was the opponent's
        // Active at the Knock Out check, another Pokémon after a switch).
        let tera = g.attacker_of_knock_out(p).and_then(|a| a.0).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_TERA)).unwrap_or(false);
        if tera && g.st.cards[me as usize].extra_prizes && prize_count > 0 {
            if let Effect::KnockOut { prize_count, .. } = g.e_mut(e) {
                *prize_count += 1;
            }
        }
        g.st.cards[me as usize].extra_prizes = false;
        // Moves this card from the knocked-out player's supporter pile (usually a no-op).
        move_cards(g, ListRef::Supporter(p as u8), ListRef::Discard(p as u8), &[me], me)?;
    }
    Ok(())
}
