//! Psyduck (ASC 39 / MEP 7): Damp — Pokémon in play (both yours and your
//! opponent's) lose any Ability that requires the Pokémon using it to Knock
//! Out itself. Ram — 20.
//!
//! Twinleaf HANDLE_ABILITY_LOCK ('remove' mode, onlyKnocksOutSelf,
//! exemptPowerNames ['Damp'], allowUseFromHand, allowUseFromDiscard, error
//! BLOCKED_BY_ABILITY): strips matching Abilities from
//! CheckPokemonPowersEffect and throws on PowerEffect when this card is in play
//! (any Pokémon card of either player, top card of a slot), the checked card is
//! in a Pokémon slot (findCardList failures count as not locked), and a real
//! PowerEffect for Damp by this card's owner doesn't throw. The generic
//! lock probe is never subject (it has no knocksOutSelf).
//! Shared with Golduck ([`reduce_damp`]).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Psyduck@ASC|MEP", mask: mask(&[k::CHECK_POKEMON_POWERS, k::POWER]), reduce, resume: None, coin: None, can_play: None };

/// `IS_POWER_SUBJECT_TO_ABILITY_LOCK` with Damp's options.
fn subject(g: &Game, power: PowerRef) -> bool {
    if power.index == PROBE_GENERIC {
        return false;
    }
    let d = &g.st.cdef(power.card).powers[power.index as usize];
    d.power_type == PowerType::Ability as u8 && d.knocks_out_self && !d.exempt_from_ability_lock && d.name != "Damp" && !d.use_from_hand && !d.use_from_discard
}

/// The `HANDLE_ABILITY_LOCK` callback.
fn is_locked(g: &mut Game, me: CardId, card: CardId) -> bool {
    // IS_ABILITY_LOCKER_IN_PLAY
    let mut in_play = false;
    for q in 0..2 {
        if for_each_pokemon(g, q, PlayerType::BottomPlayer).iter().any(|(_, c, _)| *c == me) {
            in_play = true;
        }
    }
    if !in_play {
        return false;
    }
    match g.st.locate(card) {
        Some(ListRef::Slot(..)) => {}
        _ => return false,
    }
    let owner = match g.st.locate(me).and_then(|l| l.owner()) {
        Some(o) => o,
        None => return false,
    };
    // CAN_APPLY_LOCKER_ABILITY
    g.run_fx(Effect::Power { p: owner as u8, power: PowerRef { card: me, index: 0 }, card: me, target: None, probe: false }).is_ok()
}

pub fn reduce_damp(g: &mut Game, me: CardId, e: EffId) -> R {
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
                bail!("BLOCKED_BY_ABILITY");
            }
        }
        _ => {}
    }
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    reduce_damp(g, me, e)
}
