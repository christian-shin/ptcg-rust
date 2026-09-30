//! Team Rocket's Watchtower (DRI, stadium): [C] Pokémon in play (both
//! yours and your opponent's) have no Abilities.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, allowUseFromHand and
//! allowUseFromDiscard, error BLOCKED_BY_EFFECT): strips Abilities from
//! CheckPokemonPowersEffect and throws on PowerEffect. The lock applies to a
//! card on a Pokémon slot whose CheckPokemonTypeEffect includes [C] (unless
//! stadium effects on that slot are blocked); a card not found in any list
//! falls back to its printed types.
use crate::cards::prelude::*;
use crate::engine::game_effect::pokemon_types;

pub static IMPL: CardImpl = CardImpl {
    class: "TeamRocketsWatchtower",
    mask: mask(&[k::CHECK_POKEMON_POWERS, k::POWER, k::USE_STADIUM]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

/// `IS_POWER_SUBJECT_TO_ABILITY_LOCK` with this card's options.
fn subject(g: &Game, power: PowerRef) -> bool {
    if power.index == PROBE_GENERIC {
        return true;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    d.power_type == PowerType::Ability as u8 && !d.exempt_from_ability_lock && !d.use_from_hand && !d.use_from_discard
}

fn is_locked(g: &mut Game, me: CardId, card: CardId) -> bool {
    if g.st.stadium_card() != Some(me) {
        return false;
    }
    match g.st.locate(card) {
        None => g.st.cdef(card).card_type.contains(&ct::COLORLESS),
        Some(ListRef::Slot(q, s)) => {
            let target = SlotRef::new(q as usize, s);
            if is_stadium_effect_blocked(g, q as usize, target, me) {
                return false;
            }
            let types = pokemon_types(g, target);
            match g.run_fx(Effect::CheckPokemonType { target, card_types: types }) {
                Ok((Effect::CheckPokemonType { card_types, .. }, _)) => card_types.contains(&ct::COLORLESS),
                Ok(_) => false,
                Err(_) => g.st.cdef(card).card_type.contains(&ct::COLORLESS),
            }
        }
        Some(_) => false,
    }
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckPokemonPowers { target, powers, .. } => {
            if is_locked(g, me, target) {
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
        Effect::Power { power, card, .. } => {
            if subject(g, power) && is_locked(g, me, card) {
                bail!("BLOCKED_BY_EFFECT");
            }
        }
        _ => {}
    }
    if let Effect::UseStadium { .. } = *g.e(e) {
        if g.st.stadium_card() == Some(me) {
            bail!("CANNOT_USE_STADIUM");
        }
    }
    Ok(())
}
