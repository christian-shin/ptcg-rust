//! Gastrodon (SSP): Sticky Bind — as long as this Pokémon is on your Bench,
//! Benched Stage 2 Pokémon (both yours and your opponent's) have no
//! Abilities. Mud Shot — 80.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, allowUseFromHand,
//! allowUseFromDiscard, error BLOCKED_BY_ABILITY): strips Abilities from
//! CheckPokemonPowersEffect and throws on PowerEffect when this card is the
//! top card of a Bench slot of either player, the checked card is Stage 2,
//! its list (findCardList, which throws INVALID_GAME_STATE when the card is
//! in no list) is a Pokémon slot other than either Active, and this card's
//! ability isn't blocked for the player whose Bench holds it.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Gastrodon@SSP", mask: mask(&[k::CHECK_POKEMON_POWERS, k::POWER]), reduce, resume: None, coin: None, can_play: None };

/// `IS_POWER_SUBJECT_TO_ABILITY_LOCK` with this card's options.
fn subject(g: &Game, power: PowerRef) -> bool {
    if power.index == PROBE_GENERIC {
        return true;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    d.power_type == PowerType::Ability as u8 && !d.exempt_from_ability_lock && !d.use_from_hand && !d.use_from_discard
}

fn on_bench(g: &Game, q: usize, me: CardId) -> bool {
    let pl = &g.st.players[q];
    pl.bench.iter().any(|b| g.st.slot_pokemon(q, *b) == Some(me))
}

fn is_locked(g: &mut Game, me: CardId, player: usize, card: CardId) -> R<bool> {
    let opp = 1 - player;
    let mine = on_bench(g, player, me);
    let theirs = on_bench(g, opp, me);
    if !mine && !theirs {
        return Ok(false);
    }
    if g.st.cdef(card).stage != Stage::Stage2 as u8 {
        return Ok(false);
    }
    match g.st.locate(card) {
        None => bail!("INVALID_GAME_STATE"),
        Some(ListRef::Slot(q, s)) => {
            let q = q as usize;
            if s == g.st.players[q].active {
                return Ok(false);
            }
        }
        Some(_) => return Ok(false),
    }
    let gp = if mine { player } else { opp };
    if is_ability_blocked(g, gp, me, None) {
        return Ok(false);
    }
    Ok(true)
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckPokemonPowers { p, target, powers } => {
            if is_locked(g, me, p as usize, target)? {
                let mut out = SVec::new();
                for pw in powers.iter() {
                    if !subject(g, *pw) {
                        out.push(*pw);
                    }
                }
                if let Effect::CheckPokemonPowers { powers, .. } = g.e_mut(e) {
                    *powers = out;
                }
            }
        }
        Effect::Power { p, power, card, .. } => {
            if subject(g, power) && is_locked(g, me, p as usize, card)? {
                bail!("BLOCKED_BY_ABILITY");
            }
        }
        _ => {}
    }
    Ok(())
}
